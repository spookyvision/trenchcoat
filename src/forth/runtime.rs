use serde::{Deserialize, Serialize};

use super::bytecode::{CellData, FFI, VM};

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum JSIshFFI {
    ConsoleLogStr,
}

#[derive(Debug, Clone, Default)]

pub(crate) struct MinimalRuntime {}

pub(crate) trait ConsoleRuntime {
    fn log(&self, s: impl AsRef<str>);
}

impl<VM> super::bytecode::FFI<VM> for JSIshFFI {
    fn dispatch(&self, vm: &mut VM) {
        match self {
            JSIshFFI::ConsoleLogStr => {
                let str = vm.get_str();
                vm.runtime().log(str);
            }
        }
    }
}
