// t1 =  time(.1)
// t2 = time(0.13)
// }

// export function render(index) {
// c1 = 1-abs(index - hl)/hl
// c2 = wave(c1)
// c3 = wave(c2 + t1)
// v = wave(c3 + t1)
// v = v*v
// hsv(c1 + t2,1,v)
// }

use serde::{Deserialize, Serialize};

use super::traits::{PixelBlazeRuntime, TimerMs};
use crate::forth::bytecode::{Cell, CellData, FFI, VM};

pub const PI: CellData = CellData::unwrapped_from_str("3.141592653589793");
pub const PI2: CellData = CellData::unwrapped_from_str("6.283185307179586");

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum PixelBlazeFFI {
    ConsoleLog1,
    Sin,
    Time,
    Wave,
    Abs,
    Hsv,
}

pub trait StackVM<FFI_GEN, STR> {
    fn pop(&mut self) -> CellData;
    fn get_str(&mut self) -> STR;
    fn push(&mut self, i: Cell<FFI_GEN>);
}

impl<FFI_GEN, RT, VM_GEN, STR> FFI<VM_GEN> for PixelBlazeFFI
where
    RT: PixelBlazeRuntime,
    FFI_GEN: core::fmt::Debug + Clone,
    VM_GEN: StackVM<FFI_GEN, STR>,
    STR: AsRef<str>,
{
    fn dispatch(&self, vm: &mut VM<FFI_GEN, RT>) {
        match self {
            PixelBlazeFFI::ConsoleLog1 => {
                let str = vm.get_str();
                console_log(str.as_ref());
            }
            PixelBlazeFFI::Sin => {
                vm.run().ok();
                let top = vm.pop();
                let res = cordic::sin(top.unwrap_val());
                vm.push(Cell::Val(res));
            }
            PixelBlazeFFI::Time => {
                vm.run().ok();
                let top = vm.pop();
                let res = time(top.unwrap_val(), *vm.runtime());
                vm.push(Cell::Val(res));
            }
            PixelBlazeFFI::Wave => {
                vm.run().ok();
                let top = vm.pop();
                let res = wave(top.unwrap_val());
                vm.push(Cell::Val(res));
            }
            PixelBlazeFFI::Abs => {
                vm.run().ok();
                let top = vm.pop();
                let res = abs(top.unwrap_val());
                vm.push(Cell::Val(res));
            }
            PixelBlazeFFI::Hsv => {
                vm.run().ok();
                let v = vm.pop().unwrap_val();

                vm.run().ok();
                let s = vm.pop().unwrap_val();

                vm.run().ok();
                let h = vm.pop().unwrap_val();

                hsv(h, s, v);
            }
        }
    }
}

fn console_log(s: &str) {
    println!("[VM::LOG] {s}")
}

pub(crate) fn time(interval: CellData, runtime: impl PixelBlazeRuntime) -> CellData {
    let now = CellData::from_num((runtime.time_millis() * 65) as u16);
    now * interval
}

pub(crate) fn abs(val: CellData) -> CellData {
    val.abs()
}

pub(crate) fn hsv(h: CellData, s: CellData, v: CellData) {
    println!("set hsv! {h:?}, {s:?}, {v:?}");
}

pub(crate) fn sin(val: CellData) -> CellData {
    cordic::sin(val)
}

pub(crate) fn wave(val: CellData) -> CellData {
    (CellData::from_num(1) + sin(val * PI2)) / CellData::from_num(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        forth::{bytecode::Op, util::assert_similar},
        pixelblaze::util::vm,
    };
    #[test]
    fn test_wave() {
        let decimals = 2;
        assert_similar(0.5, wave(CellData::from_num(0.0)), decimals);
        assert_similar(1.0, wave(CellData::from_num(0.25)), decimals);
        assert_similar(0.5, wave(CellData::from_num(0.5)), decimals);
        assert_similar(0.0, wave(CellData::from_num(0.75)), decimals);
    }

    #[test]
    fn test_ffi() -> anyhow::Result<()> {
        let mut vm = vm();
        vm.push(Cell::from(-5i32));
        vm.push(Op::FFI(PixelBlazeFFI::Abs).into());
        vm.run();
        assert_eq!(&[Cell::from(5i32)], &vm.stack);
        Ok(())
    }

    #[test]
    fn test_sin() -> anyhow::Result<()> {
        let mut vm = vm();

        let param = 0.1f64;
        vm.push(Cell::val(param));
        vm.push(Cell::Op(Op::FFI(PixelBlazeFFI::Sin)));

        vm.run();

        let precise: f64 = param.sin();
        let approximate = vm.stack.pop().unwrap().unwrap_val();

        assert_similar(precise, approximate, 1);
        Ok(())
    }
}
