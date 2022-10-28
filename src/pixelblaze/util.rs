use super::{ffi::PixelBlazeFFI, runtime::ConsoleRuntime};
use crate::forth::vm::{CellData, VM};

pub(crate) fn vm() -> VM<PixelBlazeFFI, ConsoleRuntime> {
    VM::new_empty(ConsoleRuntime::default())
}
