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

use super::bytecode::{CellData, TimerMs, VM};

pub const PI: CellData = CellData::unwrapped_from_str("3.141592653589793");
pub const PI2: CellData = CellData::unwrapped_from_str("6.283185307179586");

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

#[test]
fn test_wave() {
    let decimals = 2;
    super::util::assert_similar(0.5, wave(CellData::from_num(0.0)), decimals);
    super::util::assert_similar(1.0, wave(CellData::from_num(0.25)), decimals);
    super::util::assert_similar(0.5, wave(CellData::from_num(0.5)), decimals);
    super::util::assert_similar(0.0, wave(CellData::from_num(0.75)), decimals);
}

pub(crate) fn time<T, P>(interval: CellData, vm_context: &VM<T, P>) -> CellData
where
    T: TimerMs,
{
    let now = CellData::from_num((vm_context.time_millis() * 65) as u16);
    now * interval
}
