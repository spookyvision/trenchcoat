use super::bytecode::{CellData, VM};

pub fn assert_similar(expected: f64, actual: CellData, decimals: u8) {
    let fac = 10f64.powf(decimals as _);
    let actual = (actual.to_num::<f64>() * fac).round() as i32;
    let expected = (expected * fac).round() as i32;
    assert_eq!(actual, expected);
}
