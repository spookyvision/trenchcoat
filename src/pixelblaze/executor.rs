use super::traits::PixelBlazeRuntime;
use crate::forth::bytecode::{CellData, VM};

struct Exec<FFI, RT> {
    vm: VM<FFI, RT>,
    pixel_count: usize,
    last_millis: u32,
}

impl<FFI, RT> Exec<FFI, RT>
where
    RT: PixelBlazeRuntime,
    FFI: Clone + core::fmt::Debug,
{
    pub fn new(vm: VM<FFI, RT>, pixel_count: usize) -> Self {
        let last_millis = vm.runtime().time_millis();
        Self {
            vm,
            pixel_count,
            last_millis,
        }
    }

    pub fn start(&mut self) {
        let vm = &mut self.vm;
        vm.set_var("pixelCount", CellData::from_num(self.pixel_count));
        vm.dump_state();
        vm.run();
        self.last_millis = vm.runtime().time_millis();
    }

    pub fn do_frame(&mut self) {
        let vm = &mut self.vm;
        let now = vm.runtime().time_millis();
        let delta = now - self.last_millis;
        self.last_millis = now;

        vm.push(delta.into());
        vm.call_fn("beforeRender");
        vm.pop(); // toss away implicitly returned null

        for pixel_idx in 0..self.pixel_count {
            vm.push(pixel_idx.into());
            vm.call_fn("render");
            vm.pop(); // toss away implicitly returned null
        }
    }
}
