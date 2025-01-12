use super::{ffi::PixelBlazeFFI, traits::PixelBlazeRuntime};
use crate::forth::{
    util::pack,
    vm::{Cell, CellData, Op, VMError, VarStorage, VM},
};

#[derive(Clone, PartialEq)]
pub struct Executor<FFI: Eq, RT> {
    vm: Option<VM<FFI, RT>>,
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
            vm: Some(vm),
            pixel_count,
            last_millis,
        }
    }

    pub fn start(&mut self) -> Result<(), VMError> {
        let Some(vm) = self.vm.as_mut() else {
            return Err(VMError::Vanished);
        };
        vm.push(Op::PopRet.into());
        let s = "*** VM START ***\n";
        let ops = pack(s.as_bytes());
        for el in ops {
            vm.push(el);
        }
        let ffi = Op::FFI(PixelBlazeFFI::ConsoleLog);
        vm.push(Cell::from(ffi));
        vm.set_var("pixelCount", CellData::from_num(self.pixel_count));
        vm.run()?;
        self.last_millis = vm.runtime_mut().time_millis();
        Ok(())
    }

    pub fn exit(mut self) -> Result<(), VMError> {
        let Some(vm) = self.vm.as_mut() else {
            return Err(VMError::Vanished);
        };
        vm.push(Op::PopRet.into());
        let s = "*** DÖNE! ***";
        let ops = pack(s.as_bytes());
        for el in ops {
            vm.push(el);
        }
        let ffi = Op::FFI(PixelBlazeFFI::ConsoleLog);
        vm.push(Cell::from(ffi));
        vm.run()?;
        Ok(())
    }

    pub fn set_var(&mut self, name: impl AsRef<str>, val: CellData) {
        // TODO error handling instead of if let
        if let Some(vm) = self.vm.as_mut() {
            vm.set_var(name, val);
        }
    }

    pub fn on_slider(&mut self, name: impl AsRef<str>, val: f32) -> Result<(), VMError> {
        let Some(vm) = self.vm.as_mut() else {
            return Err(VMError::Vanished);
        };
        vm.push(val.into());
        vm.call_fn(name)?;
        vm.pop()?;

        Ok(())
    }

    pub fn dump_state(&self) {
        if let Some(vm) = self.vm.as_ref() {
            vm.dump_state();
        }
    }

    pub fn globals(&self) -> Option<&VarStorage> {
        self.vm.as_ref().map(|vm| vm.globals())
    }

    // TODO should return whether we're done huh
    pub fn do_frame(&mut self) -> Result<(), VMError> {
        let Some(vm) = self.vm.as_mut() else {
            return Err(VMError::Vanished);
        };
        let now = vm.runtime_mut().time_millis();
        let delta = now.wrapping_sub(self.last_millis);
        self.last_millis = now;

        // ensure we're not overflowing: beforeRender gets called with a delta value that
        // must fit inside the `CellData` fixed type
        // TODO FIXME SUCK millis is u32 but we use Fixed<16,16>
        let clamped_delta = delta.min(CellData::MAX.to_num());
        vm.push(clamped_delta.into());
        vm.call_fn("beforeRender")?;
        vm.pop()?; // toss bogus return value

        vm.runtime_mut().led_begin();
        // TODO performance:
        // - function call lookup can be memoized
        // - entire block can be moved inside vm maybe?
        for pixel_idx in 0..self.pixel_count {
            vm.runtime_mut().set_led_idx(pixel_idx);
            vm.push(pixel_idx.into());
            vm.call_fn("render")?;
            vm.pop()?; // toss away implicitly returned null
        }
        vm.runtime_mut().led_commit();

        Ok(())
    }

    pub fn pixel_count(&self) -> usize {
        self.pixel_count
    }

    pub fn runtime(&self) -> Option<&RT> {
        self.vm.as_ref().map(|vm| vm.runtime())
    }

    pub fn runtime_mut(&mut self) -> Option<&mut RT> {
        self.vm.as_mut().map(|vm| vm.runtime_mut())
    }

    pub fn set_vm(&mut self, vm: VM<PixelBlazeFFI, RT>) {
        self.vm = Some(vm);
    }

    pub fn take_vm(&mut self) -> Option<VM<PixelBlazeFFI, RT>> {
        self.vm.take()
    }
}
