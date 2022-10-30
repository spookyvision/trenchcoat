#![no_main]
#![no_std]

use stm32f4_app as _; // global logger + panicking-behavior + memory layout

#[rtic::app(
    device = stm32f4xx_hal::pac, // TODO: Replace `some_hal::pac` with the path to the PAC
    dispatchers = [EXTI3] // TODO: Replace the `FreeInterrupt1, ...` with free interrupt vectors if software tasks are used
)]
mod app {
    use core::mem::size_of_val;

    use defmt::{error, info};
    use dwt_systick_monotonic::DwtSystick;
    use fugit::RateExtU32;
    use postcard::accumulator::{CobsAccumulator, FeedResult};
    use stm32f4_app::runtime::F4Runtime;
    use stm32f4xx_hal::{otg_fs as usb, pac, prelude::*};
    use trenchcoat::{
        forth::vm::VM,
        pixelblaze::{executor::Executor, ffi::PixelBlazeFFI},
    };
    use usb::{UsbBus, UsbBusType, USB};
    use usb_device::{bus::UsbBusAllocator, prelude::*};
    use usbd_serial::SerialPort;

    const SYSCLK: u32 = 84_000_000;
    const USB_EP_SIZE: usize = 256;
    const BYTECODE_SIZE: usize = 256;

    #[monotonic(binds = SysTick, default = true)]
    type DwtMono = DwtSystick<SYSCLK>;

    #[shared]
    struct Shared {
        executor: Executor<PixelBlazeFFI, F4Runtime>,
    }

    #[local]
    struct Local {
        serial: SerialPort<'static, UsbBus<USB>>,
        usb_dev: UsbDevice<'static, UsbBusType>,
        bytecode: &'static mut CobsAccumulator<BYTECODE_SIZE>,
    }

    #[init(local = [
        ep: [u32; USB_EP_SIZE] = [0; USB_EP_SIZE],
        ibytecode: CobsAccumulator<BYTECODE_SIZE> = CobsAccumulator::new(),
        iusb_bus: Option<UsbBusAllocator<UsbBusType>> = None
        ])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        info!("init");
        let device: pac::Peripherals = cx.device;
        let mut cp = cx.core;
        cp.DWT.enable_cycle_counter();

        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(SYSCLK.Hz()).freeze();

        let mut dcb = cp.DCB;
        let dwt = cp.DWT;
        let systick = cp.SYST;

        let gpiob = device.GPIOB.split();
        let gpioa = device.GPIOA.split();

        let usb_dm = gpioa.pa11.into_alternate();
        let mut usb_dp = gpioa.pa12.into_push_pull_output();

        // force usb reset
        usb_dp.set_low();
        cortex_m::asm::delay(clocks.sysclk().to_kHz());

        let usb_dp = usb_dp.into_alternate();

        let usb = USB {
            usb_global: device.OTG_FS_GLOBAL,
            usb_device: device.OTG_FS_DEVICE,
            usb_pwrclk: device.OTG_FS_PWRCLK,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
            hclk: clocks.hclk(),
        };

        let ep = cx.local.ep;

        let usb_bus = cx.local.iusb_bus.insert(UsbBus::new(usb, ep));

        let serial = SerialPort::new(usb_bus);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Medusa Entertainment")
            .product("PB")
            .serial_number("BLZIT")
            .device_class(usbd_serial::USB_CLASS_CDC)
            .build();

        let spi = f4_peri::ws2812::spi(device.SPI2, gpiob.pb15.into_alternate(), &clocks);
        let ws = ws2812_spi::Ws2812::new(spi);

        let pixel_count = 16;
        let mut vm = VM::new_empty(F4Runtime::default());
        vm.runtime_mut().set_ws(Some(ws));
        let executor = Executor::new(vm, pixel_count);

        defmt::debug!("executor size is {}", size_of_val(&executor));

        let mono = DwtSystick::new(&mut dcb, dwt, systick, clocks.sysclk().to_Hz());
        (
            Shared { executor },
            Local {
                usb_dev,
                serial,
                bytecode: cx.local.ibytecode,
            },
            init::Monotonics(mono),
        )
    }

    // Optional idle, can be removed if not needed.
    #[idle(shared=[executor])]
    fn idle(mut cx: idle::Context) -> ! {
        info!("idle");
        cx.shared.executor.lock(|executor| {
            executor.start();
            frame::spawn().ok();
        });

        loop {
            continue;
        }
    }

    #[task(shared=[executor])]
    fn frame(mut cx: frame::Context) {
        let frame_interval_ms = 25u32;
        cx.shared.executor.lock(|executor| {
            executor.do_frame();
            if let Some(runtime) = executor.runtime_mut() {
                // TODO /10 is a hack to make things look nicer,
                // something in our calcs is probably bork
                runtime.step_ms((frame_interval_ms / 10) as i32);
            }
        });
        frame::spawn_after(frame_interval_ms.millis()).unwrap();
    }

    #[task(binds = OTG_FS, local = [usb_dev, serial, bytecode], shared=[executor])]
    fn usb_rx(mut cx: usb_rx::Context) {
        let cobs_buf = cx.local.bytecode;
        let serial = cx.local.serial;

        if cx.local.usb_dev.poll(&mut [serial]) {
            let mut buf = [0u8; 64];
            match serial.read(&mut buf) {
                Ok(count) if count > 0 => {
                    let mut window = &buf[..];

                    'cobs: while !window.is_empty() {
                        window = match cobs_buf.feed::<VM<PixelBlazeFFI, F4Runtime>>(&window) {
                            FeedResult::Consumed => break 'cobs,
                            FeedResult::OverFull(new_wind) => new_wind,
                            FeedResult::DeserError(new_wind) => new_wind,
                            FeedResult::Success { data, remaining } => {
                                let mut next_vm = data;
                                cx.shared.executor.lock(|executor| {
                                    if let Some(vm) = executor.take_vm() {
                                        let rt = vm.dismember();
                                        *next_vm.runtime_mut() = rt;
                                        executor.set_vm(next_vm);
                                        executor.start();
                                    }
                                });

                                remaining
                            }
                        };
                    }
                }
                _ => {}
            }
        }
    }
}
