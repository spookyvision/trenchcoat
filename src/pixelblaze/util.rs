use super::{funcs::PixelBlazeFFI, runtime::ConsoleRuntime, traits::PixelBlazeRuntime};
use crate::forth::bytecode::VM;

pub(crate) fn vm() -> VM<PixelBlazeFFI, ConsoleRuntime> {
    VM::new_empty(ConsoleRuntime::default())
}
