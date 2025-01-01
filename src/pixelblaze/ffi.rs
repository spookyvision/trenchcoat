use core::str::from_utf8;

use fixed::traits::{Fixed, ToFixed};
use serde::{Deserialize, Serialize};

use super::traits::PixelBlazeRuntime;
use crate::forth::{
    util::StackSlice,
    vm::{Cell, CellData, FFIOps, Param, VMError},
};

// TODO this sucks - any error here is not caught by the compiler
// e.g. forget to map "rgb" => ::Rgb, boom b0rk
// also it's a ton of work:
// - Peripherals
// - FFI_FUNCS
// - FFI enum
// - call_info
// TODO also this isn't accessible at runtime, which arguably is even more sucky
#[cfg(feature = "compiler")]
pub const FFI_FUNCS: phf::Map<&'static str, PixelBlazeFFI> = phf::phf_map! {
    "console_log" => PixelBlazeFFI::ConsoleLog,
    "sin" => PixelBlazeFFI::Sin,
    "time" => PixelBlazeFFI::Time,
    "wave" => PixelBlazeFFI::Wave,
    "hsv" => PixelBlazeFFI::Hsv,
    "rgb" => PixelBlazeFFI::Rgb,
    "ext_okhsl" => PixelBlazeFFI::ExtOkHsl,
};

pub const PI: CellData = CellData::unwrapped_from_str("3.141592653589793");
pub const PI2: CellData = CellData::unwrapped_from_str("6.283185307179586");

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum PixelBlazeFFI {
    ConsoleLog,
    Sin,
    Time,
    Wave,
    Abs,
    Hsv,
    Rgb,
    ExtOkHsl,
}

impl<RT> FFIOps<RT> for PixelBlazeFFI
where
    RT: PixelBlazeRuntime,
{
    // TODO this sucks - any error here is not caught by the compiler
    fn call_info(&self) -> &[Param] {
        match self {
            PixelBlazeFFI::ConsoleLog => &[Param::DynPacked],
            PixelBlazeFFI::Hsv => &[Param::Normal, Param::Normal, Param::Normal],
            PixelBlazeFFI::Rgb => &[Param::Normal, Param::Normal, Param::Normal],
            PixelBlazeFFI::ExtOkHsl => &[Param::Normal, Param::Normal, Param::Normal],
            _ => &[Param::Normal],
        }
    }

    fn dispatch(&self, rt: &mut RT, params: &[Cell<Self>]) -> Result<Cell<Self>, VMError> {
        let res;
        match self {
            PixelBlazeFFI::ConsoleLog => {
                let v: heapless::Vec<u8, 32> = StackSlice(params)
                    .try_into()
                    .map_err(|_| VMError::Malformed)?;
                rt.log(from_utf8(&v).map_err(|_| VMError::Malformed)?);
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

                // pb spec says h wraps between 0..1
                rt.led_hsv(h.frac(), s, v);
                res = Cell::Null;
            }

            PixelBlazeFFI::Rgb => {
                let r = CellData::try_from(&params[2])?;
                let g = CellData::try_from(&params[1])?;
                let b = CellData::try_from(&params[0])?;

                rt.led_rgb(r, g, b);
                res = Cell::Null;
            }
            PixelBlazeFFI::ExtOkHsl => {
                let h = CellData::try_from(&params[2])?;
                let s = CellData::try_from(&params[1])?;
                let l = CellData::try_from(&params[0])?;

                rt.ext_led_okhsl(h.frac(), s, l);
                res = Cell::Null;
            }
        }

        Ok(res)
    }
}

pub(crate) fn time(interval: CellData, runtime: &mut impl PixelBlazeRuntime) -> CellData {
    if interval == 0 {
        return CellData::from_num(0);
    }
    let fac = interval.wrapping_mul(CellData::MAX);
    let now = CellData::from_num((runtime.time_millis()) % 32768);
    let res = (now / fac).frac();
    res
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
        forth::{util::assert_similar, vm::Op},
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
