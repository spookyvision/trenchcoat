#![no_main]
#![no_std]

use stm32f4_app as _; // global logger + panicking-behavior + memory layout

#[rtic::app(
    device = stm32f4xx_hal::pac, // TODO: Replace `some_hal::pac` with the path to the PAC
    dispatchers = [EXTI3] // TODO: Replace the `FreeInterrupt1, ...` with free interrupt vectors if software tasks are used
)]
mod app {
    use defmt::info;
    use dwt_systick_monotonic::DwtSystick;
    use fugit::RateExtU32;
    use smart_leds::RGB8;
    use stm32f4_app::runtime::F4Runtime;
    use stm32f4xx_hal::{
        dma::{config::DmaConfig, PeripheralToMemory, Stream0, StreamsTuple, Transfer},
        pac::{self, ADC1, DMA2},
        prelude::*,
    };
    use trenchcoat::{
        forth::vm::VM,
        pixelblaze::{executor::Executor, ffi::PixelBlazeFFI},
    };

    const SYSCLK: u32 = 84_000_000;
    const BYTECODE_BUF: usize = 512;

    #[monotonic(binds = SysTick, default = true)]
    type DwtMono = DwtSystick<SYSCLK>;

    // Shared resources go here
    #[shared]
    struct Shared {
        bytecode: [u8; BYTECODE_BUF],
        executor: Executor<PixelBlazeFFI, F4Runtime>,
    }

    // Local resources go here
    #[local]
    struct Local {
        // TODO: Add resources
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");
        let device: pac::Peripherals = cx.device;
        let mut cp = cx.core;
        cp.DWT.enable_cycle_counter();

        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(SYSCLK.Hz()).freeze();

        let mut dcb = cp.DCB;
        let dwt = cp.DWT;
        let systick = cp.SYST;
        let mono = DwtSystick::new(&mut dcb, dwt, systick, clocks.sysclk().to_Hz());

        let gpiob = device.GPIOB.split();
        let spi = f4_peri::ws2812::spi(device.SPI2, gpiob.pb15.into_alternate(), &clocks);
        let ws = ws2812_spi::Ws2812::new(spi);
        let vm_bytes = include_bytes!("../../../res/rainbow melt.tcb");
        let mut bytecode = [0; BYTECODE_BUF];
        bytecode[0..vm_bytes.len()].copy_from_slice(vm_bytes);
        let mut vm: VM<PixelBlazeFFI, F4Runtime> =
            postcard::from_bytes_cobs(&mut bytecode).unwrap();
        vm.runtime_mut().set_ws(Some(ws));
        vm.runtime_mut().leds_mut()[0] = RGB8::new(10, 20, 40);
        vm.runtime_mut().leds_mut()[1] = RGB8::new(100, 20, 40);

        let pixel_count = 16;
        let mut executor = Executor::new(vm, pixel_count);

        defmt::debug!("executor size is {}", core::mem::size_of_val(&executor));

        (
            Shared { bytecode, executor },
            Local {},
            init::Monotonics(mono),
        )
    }

    // Optional idle, can be removed if not needed.
    #[idle(shared=[executor])]
    fn idle(mut cx: idle::Context) -> ! {
        defmt::info!("idle");
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
            let runtime = executor.runtime_mut();
            // TODO /10 is a hack to make things look nicer,
            // something in our calcs is probably bork
            runtime.step_ms((frame_interval_ms / 10) as i32);
        });
        frame::spawn_after(frame_interval_ms.millis()).unwrap();
    }
}
