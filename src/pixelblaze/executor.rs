use super::{ffi::PixelBlazeFFI, traits::PixelBlazeRuntime};
use crate::forth::{
    util::pack,
    vm::{Cell, CellData, FFIOps, Op, VM},
};

pub struct Executor<FFI, RT> {
    vm: VM<FFI, RT>,
    pixel_count: usize,
    last_millis: u32,
}

impl<RT> Executor<PixelBlazeFFI, RT>
where
    RT: PixelBlazeRuntime,
{
    pub fn new(mut vm: VM<PixelBlazeFFI, RT>, pixel_count: usize) -> Self {
        let last_millis = vm.runtime_mut().time_millis();
        Self {
            vm,
            pixel_count,
            last_millis,
        }
    }

    pub fn start(&mut self) {
        let vm = &mut self.vm;
        vm.push(Op::PopRet.into());
        let s = "*** VM START ***\n";
        let ops = pack(s.as_bytes());
        for el in ops {
            vm.push(el);
        }
        let ffi = Op::FFI(PixelBlazeFFI::ConsoleLog);
        vm.push(Cell::from(ffi));
        vm.set_var("pixelCount", CellData::from_num(self.pixel_count));
        vm.dump_state();
        vm.run();
        self.last_millis = vm.runtime_mut().time_millis();
    }

    pub fn exit(mut self) {
        let vm = &mut self.vm;
        vm.push(Op::PopRet.into());
        let s = "*** DÖNE! ***";
        let ops = pack(s.as_bytes());
        for el in ops {
            vm.push(el);
        }
        let ffi = Op::FFI(PixelBlazeFFI::ConsoleLog);
        vm.push(Cell::from(ffi));
        vm.run();
    }

    pub fn do_frame(&mut self) {
        let vm = &mut self.vm;
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
