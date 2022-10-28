use super::{ffi::PixelBlazeFFI, runtime::ConsoleRuntime, traits::PixelBlazeRuntime};
use crate::forth::vm::{CellData, VM};

pub fn cook_hue(h: CellData) -> CellData {
    let parts: i32 = h.to_num();
    h
}
