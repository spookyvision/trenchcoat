use super::traits::PixelBlazeRuntime;
use crate::forth::bytecode::{CellData, FFIOps, VM};

pub struct Executor<FFI, RT> {
    vm: VM<FFI, RT>,
    pixel_count: usize,
    last_millis: u32,
}

impl<FFI, RT> Executor<FFI, RT>
where
    RT: PixelBlazeRuntime,
    FFI: FFIOps<RT>,
{
    pub fn new(mut vm: VM<FFI, RT>, pixel_count: usize) -> Self {
        let last_millis = vm.runtime_mut().time_millis();
        Self {
            vm,
            pixel_count,
            last_millis,
        }
    }

    pub fn start(&mut self) {
        println!("\n\n\n*** VM START ***\n");
        let mut vm = &mut self.vm;
        vm.set_var("pixelCount", CellData::from_num(self.pixel_count));
        vm.dump_state();
        vm.run();
        self.last_millis = vm.runtime_mut().time_millis();
    }

    pub fn do_frame(&mut self) {
        let mut vm = &mut self.vm;
        let now = vm.runtime_mut().time_millis();
        let delta = now - self.last_millis;
        self.last_millis = now;

        vm.push(delta.into());
        vm.call_fn("beforeRender");
        vm.pop_unchecked(); // toss away implicitly returned null

        vm.runtime_mut().led_begin();
        for pixel_idx in 0..self.pixel_count {
            vm.runtime_mut().set_led_idx(pixel_idx);
            vm.push(pixel_idx.into());
            vm.call_fn("render");
            vm.pop_unchecked(); // toss away implicitly returned null
        }
        vm.runtime_mut().led_commit();
    }
}
