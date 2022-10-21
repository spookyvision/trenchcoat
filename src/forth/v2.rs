use core::str::from_utf8;
use std::{fmt::Debug, time::Instant};

use fixed::{traits::ToFixed, types::extra::U8, FixedI32};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type CellData = FixedI32<U8>;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Cell<FFI> {
    Val(CellData),
    Op(Op<FFI>),
    Null,
}

impl<FFI> TryFrom<&Cell<FFI>> for CellData {
    type Error = VMError;

    fn try_from(value: &Cell<FFI>) -> Result<Self, Self::Error> {
        match value {
            Cell::Val(val) => Ok(*val),
            _ => Err(VMError::TypeCoercion),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]

pub enum Op<FFI> {
    Add,
    FFI(FFI),
}

pub trait FFIOps<RT>: Sized {
    fn dispatch(&self, rt: &RT, params: &[Cell<Self>]) -> Result<Cell<Self>, VMError>;
    fn call_info(&self) -> &[Param];
}

pub enum Param {
    Normal,
    DynSized,
}

struct VM<FFI, RT> {
    runtime: RT,
    stack: Vec<Cell<FFI>>,
}

impl<FFI, RT> VM<FFI, RT>
where
    FFI: Debug + FFIOps<RT>,
{
    fn new(runtime: RT, stack: Vec<Cell<FFI>>) -> Self {
        Self { runtime, stack }
    }

    pub fn push(&mut self, op: Op<FFI>) {
        println!("pushing op! {op:?}")
    }
    pub fn pop(&mut self) -> Option<Cell<FFI>> {
        self.stack.pop()
    }

    pub fn run(&mut self) {
        println!("run! {:?}", self.stack);
        while let Some(Cell::Op(op)) = self.stack.pop() {
            println!("eval {op:?}");
            self.eval(op);
            println!("{:?}", self.stack);
        }
    }

    fn eval(&mut self, op: Op<FFI>) -> Result<(), VMError> {
        match op {
            Op::Add => todo!(),
            Op::FFI(ffi) => {
                let stack_end = self.stack.len();
                let mut stack_start = stack_end;
                for param in ffi.call_info() {
                    match param {
                        Param::Normal => stack_start -= 1,
                        Param::DynSized => todo!(),
                    }
                }
                let res = ffi
                    .dispatch(&self.runtime, &self.stack[stack_start..stack_end])
                    .map_err(|err| VMError::from(err))?;
                self.stack.truncate(stack_start);
                self.stack.push(res);
            }
        }
        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
enum PureJSFFI {
    ConsoleLog,
    MathPow,
}

trait Runtime {
    fn time_millis(&self) -> u32;
}

struct StdRuntime {
    start: Instant,
}

impl StdRuntime {
    fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Runtime for StdRuntime {
    fn time_millis(&self) -> u32 {
        // don't run this on a Boeing 787
        Instant::now().duration_since(self.start).as_millis() as u32
    }
}

#[derive(Debug, Error)]
pub enum FFIError {
    #[error("wrong number of arguments")]
    NumArgs,
}

#[derive(Debug, Error)]

pub enum VMError {
    #[error("type coercion failed")]
    TypeCoercion,
    #[error("FFI bork")]
    FFI(#[from] FFIError),
}

impl<RT> FFIOps<RT> for PureJSFFI
where
    RT: Runtime,
{
    fn dispatch(&self, rt: &RT, params: &[Cell<Self>]) -> Result<Cell<Self>, VMError> {
        match self {
            PureJSFFI::ConsoleLog => {
                println!("{}", rt.time_millis());
                Ok(Cell::Null)
            }
            PureJSFFI::MathPow => {
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
            PureJSFFI::ConsoleLog => &[Param::Normal],
            PureJSFFI::MathPow => &[Param::Normal, Param::Normal],
        }
    }
}

fn main() {
    println!("*** VM START ***");
    let mut vm = VM::new(
        StdRuntime::new(),
        vec![
            Cell::Val(2.to_fixed()),
            Cell::Val(5.to_fixed()),
            Cell::Op(Op::FFI(PureJSFFI::MathPow)),
        ],
    );
    vm.run();
}
