#![no_main]
#![no_std]

use stm32f4_app as _; // global logger + panicking-behavior + memory layout

#[rtic::app(
    device = stm32f4xx_hal::pac, // TODO: Replace `some_hal::pac` with the path to the PAC
    dispatchers = [EXTI3] // TODO: Replace the `FreeInterrupt1, ...` with free interrupt vectors if software tasks are used
)]
mod app {
    use core::mem::size_of_val;

    use bbqueue::{BBBuffer, Consumer, Producer};
    use defmt::{error, info};
    use dwt_systick_monotonic::DwtSystick;
    use fugit::RateExtU32;
    use heapless::Vec;
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
    const USB_EP_SIZE: usize = 512;
    const BYTECODE_SIZE: usize = 256;

    #[monotonic(binds = SysTick, default = true)]
    type DwtMono = DwtSystick<SYSCLK>;

    #[shared]
    struct Shared {
        executor: Executor<PixelBlazeFFI, F4Runtime>,
        bytecode: Vec<u8, BYTECODE_SIZE>,
    }

    #[local]
    struct Local {
        serial: SerialPort<'static, UsbBus<USB>>,
        usb_dev: UsbDevice<'static, UsbBusType>,
        tx: Producer<'static, BYTECODE_SIZE>,
        rx: Consumer<'static, BYTECODE_SIZE>,
    }

    #[init(local = [
        ep: [u32; USB_EP_SIZE] = [0; USB_EP_SIZE],
        bb: BBBuffer<BYTECODE_SIZE> = BBBuffer::new(),
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

        let (tx, rx) = cx.local.bb.try_split().unwrap();

        let mono = DwtSystick::new(&mut dcb, dwt, systick, clocks.sysclk().to_Hz());
        (
            Shared {
                executor,
                bytecode: Vec::new(),
            },
            Local {
                usb_dev,
                serial,
                tx,
                rx,
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
            swap::spawn_after(3000.millis()).unwrap();
        });

        loop {
            continue;
        }
    }

    #[task(shared=[executor])]
    fn swap(mut cx: swap::Context) {
        // info!("swob");
        // let ser = include_bytes!("../../../res/rainbow melt low.tcb");

        // let bytecode = cx.local.bytecode;
        // bytecode[0..ser.len()].copy_from_slice(ser);
        // let mut next_vm: VM<PixelBlazeFFI, F4Runtime> =
        //     postcard::from_bytes_cobs(bytecode).unwrap();
        // cx.shared.executor.lock(|executor| {
        //     if let Some(vm) = executor.take_vm() {
        //         let rt = vm.dismember();
        //         *next_vm.runtime_mut() = rt;

        //         swap2::spawn(next_vm).ok();
        //     }
        // });
    }

    #[task(shared=[executor])]
    fn swap2(mut cx: swap2::Context, next_vm: VM<PixelBlazeFFI, F4Runtime>) {
        info!("swobbbbb");
        cx.shared.executor.lock(|executor| {
            executor.set_vm(next_vm);
            executor.start();
        });
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

    #[task(shared=[executor, bytecode], local=[])]
    fn decode(cx: decode::Context) {
        let shared = cx.shared;
        (shared.bytecode, shared.executor).lock(|bytecode, executor| {
            let mut wipe = false;
            match postcard::from_bytes_cobs::<VM<PixelBlazeFFI, F4Runtime>>(bytecode) {
                Ok(mut next_vm) => {
                    wipe = true;
                    info!("got vm!");
                    if let Some(vm) = executor.take_vm() {
                        let rt = vm.dismember();
                        *next_vm.runtime_mut() = rt;
                        executor.set_vm(next_vm);
                        executor.start();
                    }
                }
                Err(e) => error!("{}", e),
            };
            if wipe {
                bytecode.truncate(0);
            }
        });
    }

    #[task(binds = OTG_FS, local = [usb_dev, serial, tx], shared=[bytecode])]
    fn usb_rx(mut cx: usb_rx::Context) {
        static mut BUF: [u8; 32] = [0u8; 32];
        let serial = cx.local.serial;

        if cx.local.usb_dev.poll(&mut [serial]) {
            let mut buf = [0u8; 64];
            match serial.read(&mut buf) {
                Ok(count) if count > 0 => {
                    cx.shared.bytecode.lock(|bytecode| {
                        if let Err(_) = bytecode.extend_from_slice(&buf[0..count]) {
                            error!("bytecode buffer overflow");
                        } else {
                            info!("read {} bytes >> {}", count, bytecode.len());
                            decode::spawn().unwrap();
                        }
                    });
                    // cx.local.tx.grant_exact(count).map(|mut wgr| {
                    //     wgr.buf().copy_from_slice(&buf[0..count]);
                    //     wgr.commit(count);
                    // });
                }
                _ => {}
            }
        }
    }
}
