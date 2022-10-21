use super::{
    funcs::PixelBlazeFFI,
    std::{Runtime, StdTimer},
    traits::PixelBlazeRuntime,
};
use crate::forth::bytecode::VM;

pub fn vm() -> VM<PixelBlazeFFI, Runtime> {
    VM::new(Runtime::default())
}
