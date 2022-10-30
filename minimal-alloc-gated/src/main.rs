// miniature `alloc`-enabled model of the core architecture. For reference purposes only.

use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use fixed::{types::extra::U16, FixedI32};
use serde::{Deserialize, Serialize};

extern crate alloc;

pub type VarString = String;
pub type Map<K, V, const N: usize> = std::collections::HashMap<K, V>;
pub type VMVec<T, const N: usize> = alloc::vec::Vec<T>;
pub type Stack<FFI, const N: usize> = alloc::vec::Vec<Cell<FFI>>;
pub type CellData = FixedI32<U16>;
pub type VarStorage = Map<VarString, Option<CellData>, 32>;

pub type DefaultFuncDef<FFI> = Map<VarString, FuncDef<FFI>, 4>;
pub type DefaultStack<FFI> = Stack<FFI, 64>;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Cell<FFI> {
    Op(Op<FFI>),
    Null,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Op<FFI> {
    Return, // data stack -> return stack
    FFI(FFI),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FuncDef<FFI> {
    params: VMVec<VarString, 4>,
    stack: DefaultStack<FFI>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct VM<FFI, RT>
where
    FuncDef<FFI>: PartialEq,
    FFI: Eq,
{
    stack: DefaultStack<FFI>,
    return_stack: Stack<FFI, 4>,
    return_addr: Option<usize>,
    globals: VarStorage,
    locals: VMVec<VarStorage, 8>,
    funcs: DefaultFuncDef<FFI>,
    #[serde(skip)]
    runtime: RT,
}

pub trait FFIOps<RT>: Sized + Clone + Debug {}

impl<FFI, RT> VM<FFI, RT>
where
    FFI: FFIOps<RT> + Eq,
    FuncDef<FFI>: PartialEq,
{
    pub fn new_empty(runtime: RT) -> Self {
        Self {
            stack: Default::default(),
            return_stack: Default::default(),
            return_addr: None,
            globals: Map::new(),
            locals: Default::default(),
            funcs: DefaultFuncDef::new(),
            runtime,
        }
    }
}

fn main() {
    let vm: VM<NoFFI, _> = VM::new_empty(MockRuntime);
    let ser = serde_json::to_string(&vm);
}

// TODO medium sized wart
#[derive(Clone, PartialEq, Default)]
pub struct MockRuntime;

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Debug)]

pub struct NoFFI;

pub trait NoRuntime {}

impl NoRuntime for MockRuntime {}

impl<RT> FFIOps<RT> for NoFFI where RT: NoRuntime {}
