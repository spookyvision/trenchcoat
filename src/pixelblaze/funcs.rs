use phf::phf_map;
use serde::{Deserialize, Serialize};

use super::traits::PixelBlazeRuntime;
use crate::forth::bytecode::{Cell, CellData, FFIOps, Param, VMError};

pub const FFI_FUNCS: phf::Map<&'static str, PixelBlazeFFI> = phf_map! {
    "console_log" => PixelBlazeFFI::ConsoleLog1,
    "sin" => PixelBlazeFFI::Sin,
    "time" => PixelBlazeFFI::Time,
    "wave" => PixelBlazeFFI::Wave,
    "hsv" => PixelBlazeFFI::Hsv,
};

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

impl<RT> FFIOps<RT> for PixelBlazeFFI
where
    RT: PixelBlazeRuntime,
{
    fn call_info(&self) -> &[Param] {
        match self {
            PixelBlazeFFI::ConsoleLog1 => &[Param::DynPacked],
            PixelBlazeFFI::Hsv => &[Param::Normal, Param::Normal, Param::Normal],
            _ => &[Param::Normal],
        }
    }

    fn dispatch(&self, rt: &mut RT, params: &[Cell<Self>]) -> Result<Cell<Self>, VMError> {
        let res;
        match self {
            PixelBlazeFFI::ConsoleLog1 => {
                let str = "TODO fwd to CoreRuntime somehow?";
                rt.log(str.as_ref());
                res = Cell::Null;
            }
            PixelBlazeFFI::Sin => {
                let angle = CellData::try_from(&params[0])?;
                let inner_res = cordic::sin(angle);
                res = Cell::Val(inner_res);
            }
            PixelBlazeFFI::Time => {
                let interval = CellData::try_from(&params[0])?;
                let inner_res = time(interval, rt);
                res = Cell::Val(inner_res);
            }
            PixelBlazeFFI::Wave => {
                let arg = CellData::try_from(&params[0])?;
                let inner_res = wave(arg);
                res = Cell::Val(inner_res);
            }
            PixelBlazeFFI::Abs => {
                let arg = CellData::try_from(&params[0])?;
                let inner_res = abs(arg);
                res = Cell::Val(inner_res);
            }
            PixelBlazeFFI::Hsv => {
                let h = CellData::try_from(&params[2])?;
                let s = CellData::try_from(&params[1])?;
                let v = CellData::try_from(&params[0])?;
                rt.led_hsv(h, s, v);
                res = Cell::Null;
            }
        }

        Ok(res)
    }
}

fn console_log(s: &str) {
    println!("[VM::LOG] {s}")
}

pub(crate) fn time(interval: CellData, runtime: &mut impl PixelBlazeRuntime) -> CellData {
    let now = CellData::from_num((runtime.time_millis() % 1000) * 65);
    (now * interval) / 1000
}

pub(crate) fn abs(val: CellData) -> CellData {
    val.abs()
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
    fn test_abs() -> anyhow::Result<()> {
        let mut vm = vm();
        vm.push(Cell::from(-5i32));
        vm.push(Op::FFI(PixelBlazeFFI::Abs).into());
        vm.run();
        assert_eq!(&[Cell::from(5i32)], vm.stack());
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
        let approximate = vm.pop_unchecked().unwrap_val();

        assert_similar(precise, approximate, 1);
        Ok(())
    }
}
