#![no_main]
#![no_std]

use stm32f4_app as _; // global logger + panicking-behavior + memory layout

#[rtic::app(
    device = stm32f4xx_hal::pac, // TODO: Replace `some_hal::pac` with the path to the PAC
    dispatchers = [EXTI3] // TODO: Replace the `FreeInterrupt1, ...` with free interrupt vectors if software tasks are used
)]
mod app {
    use core::mem::size_of_val;

    use defmt::info;
    use dwt_systick_monotonic::DwtSystick;
    use fugit::RateExtU32;
    use stm32f4_app::runtime::F4Runtime;
    use stm32f4xx_hal::{pac, prelude::*};
    use trenchcoat::{
        forth::vm::VM,
        pixelblaze::{executor::Executor, ffi::PixelBlazeFFI},
    };

    const SYSCLK: u32 = 84_000_000;
    const BYTECODE_BUF: usize = 256;

    #[monotonic(binds = SysTick, default = true)]
    type DwtMono = DwtSystick<SYSCLK>;

    #[shared]
    struct Shared {
        executor: Executor<PixelBlazeFFI, F4Runtime>,
    }

    #[local]
    struct Local {
        bytecode: [u8; BYTECODE_BUF],
    }

    #[init]
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
        let mono = DwtSystick::new(&mut dcb, dwt, systick, clocks.sysclk().to_Hz());

        let gpiob = device.GPIOB.split();
        let spi = f4_peri::ws2812::spi(device.SPI2, gpiob.pb15.into_alternate(), &clocks);
        let ws = ws2812_spi::Ws2812::new(spi);

        let mut bytecode = [0; BYTECODE_BUF];

        let ser = include_bytes!("../../../res/rainbow melt.tcb");
        bytecode[0..ser.len()].copy_from_slice(ser);
        let mut vm: VM<PixelBlazeFFI, F4Runtime> =
            postcard::from_bytes_cobs(&mut bytecode).unwrap();
        vm.runtime_mut().set_ws(Some(ws));

        let pixel_count = 16;
        let executor = Executor::new(vm, pixel_count);

        defmt::debug!("executor size is {}", size_of_val(&executor));

        (
            Shared { executor },
            Local { bytecode },
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

    #[task(shared=[executor], local=[bytecode])]
    fn swap(mut cx: swap::Context) {
        info!("swob");
        let ser = include_bytes!("../../../res/rainbow melt low.tcb");

        let bytecode = cx.local.bytecode;
        bytecode[0..ser.len()].copy_from_slice(ser);
        let mut next_vm: VM<PixelBlazeFFI, F4Runtime> =
            postcard::from_bytes_cobs(bytecode).unwrap();
        cx.shared.executor.lock(|executor| {
            if let Some(vm) = executor.take_vm() {
                let rt = vm.dismember();
                *next_vm.runtime_mut() = rt;

                swap2::spawn(next_vm).ok();
            }
        });
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
}
