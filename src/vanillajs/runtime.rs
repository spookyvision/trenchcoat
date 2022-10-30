use core::str::from_utf8;

use fixed::traits::ToFixed;
use serde::{Deserialize, Serialize};

use crate::forth::{
    util::StackSlice,
    vm::{Cell, CellData, FFIError, FFIOps, Param, VMError},
};
pub trait VanillaJSRuntime {
    fn time_millis(&mut self) -> u32;
    fn log(&mut self, s: &str);
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum VanillaJSFFI {
    ConsoleLog,
    MathPow,
}

impl<RT> FFIOps<RT> for VanillaJSFFI
where
    RT: VanillaJSRuntime,
{
    fn dispatch(&self, rt: &mut RT, params: &[Cell<Self>]) -> Result<Cell<Self>, VMError> {
        match self {
            VanillaJSFFI::ConsoleLog => {
                let v: heapless::Vec<u8, 32> = StackSlice(params)
                    .try_into()
                    .map_err(|_| VMError::Malformed)?;
                rt.log(from_utf8(&v).map_err(|_| VMError::Malformed)?);
                Ok(Cell::Null)
            }
            VanillaJSFFI::MathPow => {
                if params.len() != 2 {
                    return Err(FFIError::NumArgs.into());
                }
                let p1: i32 = CellData::try_from(&params[0])?.to_num();
                let p2: i32 = CellData::try_from(&params[1])?.to_num();
                let res = p1.pow(p2 as u32);
                Ok(Cell::Val(res.to_fixed()))
            }
        }
    }

    fn call_info(&self) -> &[Param] {
        match self {
            VanillaJSFFI::ConsoleLog => &[Param::DynPacked],
            VanillaJSFFI::MathPow => &[Param::Normal, Param::Normal],
        }
    }
}

#[cfg(any(test, feature = "use-std"))]
pub mod stud {
    use std::time::Instant;

    use super::*;

    pub struct StdRuntime {
        start: Instant,
    }

    impl StdRuntime {
        pub fn new() -> Self {
            Self {
                start: Instant::now(),
            }
        }
    }

    impl Default for StdRuntime {
        fn default() -> Self {
            Self::new()
        }
    }

    impl VanillaJSRuntime for StdRuntime {
        fn time_millis(&mut self) -> u32 {
            // don't run this on a Boeing 787
            self.start.elapsed().as_millis() as u32
        }

        fn log(&mut self, s: &str) {
            println!("{s}");
        }
    }

    pub struct TestRuntime {
        start: Instant,
        last_log: Option<String>,
    }

    impl TestRuntime {
        pub fn new() -> Self {
            Self {
                start: Instant::now(),
                last_log: None,
            }
        }

        pub fn last_log(&self) -> Option<&str> {
            self.last_log.as_deref()
        }
    }

    impl VanillaJSRuntime for TestRuntime {
        fn time_millis(&mut self) -> u32 {
            self.start.elapsed().as_millis() as u32
        }

        fn log(&mut self, s: &str) {
            self.last_log = Some(s.to_string())
        }
    }
}

#[cfg(any(test, feature = "use-std"))]
pub use stud::StdRuntime;

#[cfg(feature = "compiler")]
impl VanillaJSRuntime for crate::forth::compiler::MockRuntime {
    fn time_millis(&mut self) -> u32 {
        unimplemented!()
    }

    fn log(&mut self, s: &str) {
        unimplemented!()
    }
}
