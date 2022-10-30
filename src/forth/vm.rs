use core::{fmt::Debug, marker::PhantomData};

use fixed::{traits::ToFixed, types::extra::U16, FixedI32};
#[cfg(not(feature = "alloc"))]
use heapless::Entry;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[cfg(not(feature = "alloc"))]
pub(crate) mod types {

    pub type VarString = heapless::String<16>;
    pub type Map<K, V, const N: usize> = heapless::FnvIndexMap<K, V, N>;

    pub type Stack<FFI, const N: usize> = heapless::Vec<Cell<FFI>, N>;

    pub type VMVec<T, const N: usize> = heapless::Vec<T, N>;
}

#[cfg(feature = "alloc")]
use std::collections::hash_map::Entry;
#[cfg(feature = "alloc")]
pub(crate) mod types {

    use super::Cell;

    pub type VarString = String;
    pub type Map<K, V, const N: usize> = std::collections::HashMap<K, V>;

    pub type Stack<FFI, const N: usize> = alloc::vec::Vec<Cell<FFI>>;
    pub type VMVec<T, const N: usize> = alloc::vec::Vec<T>;
}

use types::*;
pub use types::*;

pub type DefaultStack<FFI> = Stack<FFI, 64>;

// TODO pixelblaze uses <16,16> but that's not the best general range
// -> definitely feature gate this to at least have <24,8>
// -> bite the `f32` bullet?
// (strict JS compliance would need `f64`, also "bitwise operations will convert it to a 32 bit integer."
// https://www.ecma-international.org/publications/files/ECMA-ST/Ecma-262.pdf

pub type CellData = FixedI32<U16>;
pub type VarStorage = Map<VarString, Option<CellData>, 32>;

pub type DefaultFuncDef<FFI> = Map<VarString, FuncDef<FFI>, 4>;
impl<FFI> TryFrom<&Cell<FFI>> for CellData {
    type Error = VMError;

    fn try_from(value: &Cell<FFI>) -> Result<Self, Self::Error> {
        match value {
            Cell::Val(val) => Ok(*val),
            _ => Err(VMError::TypeCoercion),
        }
    }
}

pub trait FFIOps<RT>: Sized + Clone + Debug {
    fn dispatch(&self, rt: &mut RT, params: &[Cell<Self>]) -> Result<Cell<Self>, VMError>;
    fn call_info(&self) -> &[Param];
}

pub enum Param {
    Normal,
    DynPacked,
}

#[cfg_attr(feature = "use-std", derive(thiserror::Error))]
#[derive(Debug, Serialize, Deserialize)]
pub enum FFIError {
    #[cfg_attr(feature = "use-std", error("Function not found"))]
    FunctionNotFound,
    #[cfg_attr(feature = "use-std", error("Wrong number of arguments"))]
    NumArgs,
}

#[cfg_attr(feature = "use-std", derive(thiserror::Error))]
#[derive(Debug, Serialize, Deserialize)]

pub enum VMError {
    #[cfg_attr(feature = "use-std", error("FixmeNotAnErrorExhausted"))]
    FixmeNotAnErrorExhausted,
    #[cfg_attr(feature = "use-std", error("type coercion failed"))]
    TypeCoercion,
    #[cfg_attr(feature = "use-std", error("FFI bork"))]
    // #[cfg(feature = "use-std")]
    FFI(#[cfg_attr(feature = "use-std", from)] FFIError),
    // #[cfg(not(feature = "use-std"))]
    // FFI(FFIError),
    #[cfg_attr(feature = "use-std", error("Malformed stack"))]
    Malformed,
    #[cfg_attr(feature = "use-std", error("Stack underflow"))]
    Underflow,
    #[cfg_attr(feature = "use-std", error("Stack overflow"))]
    Overflow,
}

#[cfg(not(feature = "use-std"))]
impl From<FFIError> for VMError {
    fn from(value: FFIError) -> Self {
        VMError::FFI(value)
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Op<FFI> {
    Return, // data stack -> return stack
    Nruter, // return stack -> data stack
    ExitFn, // TODO never used, remove/change?
    PopRet, // pop return stack
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Call(VarString),
    DeclVar(VarString),
    SetVar(VarString),
    GetVar(VarString),
    FFI(FFI),
}

type BinOp = fn(CellData, CellData) -> CellData;

fn err(s: &str) {
    panic!("ERR: {s}")
}

// TODO use Option<Cell> instead of `Cell::Null`?
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Cell<FFI> {
    Val(CellData),
    Raw(i32),
    Op(Op<FFI>),
    Null,
}

impl<TF, FFI> From<TF> for Cell<FFI>
where
    TF: ToFixed,
{
    fn from(val: TF) -> Self {
        Self::Val(CellData::from_num(val))
    }
}

impl<FFI> From<Op<FFI>> for Cell<FFI> {
    fn from(op: Op<FFI>) -> Self {
        Self::Op(op)
    }
}

impl<FFI> Cell<FFI> {
    pub(crate) fn val(num: impl ToFixed) -> Self {
        Self::Val(num.to_fixed())
    }
    pub(crate) fn unwrap_val(&self) -> CellData {
        match self {
            Cell::Val(val) => *val,
            Cell::Op(_) => panic!("tried to read value but found op"),
            Cell::Raw(_) => panic!("tried to read raw"),
            Cell::Null => panic!("tried to read null"),
        }
    }

    pub(crate) fn unwrap_raw(&self) -> i32 {
        match self {
            Cell::Raw(val) => *val,
            _ => panic!("tried to read !Raw"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FuncDef<FFI> {
    params: VMVec<VarString, 4>,
    stack: DefaultStack<FFI>,
    _phantom: PhantomData<FFI>,
}

impl<FFI> FuncDef<FFI> {
    pub fn new<P: AsRef<str>>(params: &[P], stack: Stack<FFI, 64>) -> Self {
        let mut our_params = VMVec::new();
        for param in params {
            our_params.push(param.as_ref().into());
        }
        Self {
            stack,
            params: our_params,
            _phantom: PhantomData,
        }
    }

    pub fn stack(&self) -> &[Cell<FFI>] {
        self.stack.as_ref()
    }

    pub fn params(&self) -> &[impl AsRef<str>] {
        self.params.as_slice()
    }
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

    pub fn new(stack: DefaultStack<FFI>, funcs: DefaultFuncDef<FFI>, runtime: RT) -> Self {
        Self {
            stack,
            return_stack: Default::default(),
            return_addr: None,
            globals: Map::new(),
            locals: Default::default(),
            funcs,
            runtime,
        }
    }

    pub fn dismember(self) -> RT {
        self.runtime
    }

    pub fn dump_state(&self) {
        trench_debug!("stack: {:?}", self.stack);
        trench_debug!("rstack: {:?}", self.return_stack);
        trench_debug!("globals: {:?}", self.globals);
        trench_debug!("locals: {:?}", self.locals);
        let debug_funcs = true;
        if debug_funcs {
            for (name, def) in &self.funcs {
                trench_debug!("F {} => {:?}", name, def)
            }
        }
    }

    fn binary_op(&mut self, op: BinOp) {
        // TODO error propagation ("Exhausted" should not be an error...)
        // trench_debug!("\n\n\n\n---bop\n");
        self.dump_state();
        self.run().ok();
        self.dump_state();
        let y = self.pop_unchecked().unwrap_val();
        self.run().ok();
        let x = self.pop_unchecked().unwrap_val();

        self.push(Cell::Val(op(x, y)));
    }

    fn eval(&mut self, op: &Op<FFI>) -> Result<(), VMError> {
        // trench_debug!("----");
        // trench_debug!("eval {self:?}");
        match op {
            Op::ExitFn => {
                self.exit_fn();
            }
            Op::PopRet => {
                self.return_stack.pop().ok_or(VMError::Underflow)?;
            }
            Op::Return => {
                self.do_return();
            }
            // TODO: test
            Op::Nruter => {
                let cell = self.return_stack.pop().ok_or(VMError::Underflow)?;
                if self.stack.capacity() == 0 {
                    return Err(VMError::Overflow);
                }
                self.stack.push(cell);
            }
            Op::Call(name) => {
                self.call_fn(name);
            }
            Op::Add => self.binary_op(|x, y| x + y),
            Op::Sub => self.binary_op(|x, y| x - y),
            Op::Mul => self.binary_op(|x, y| x * y),
            Op::Div => self.binary_op(|x, y| x / y),
            Op::Mod => self.binary_op(|x, y| x % y),
            Op::And => self.binary_op(|x, y| x & y),
            Op::Or => self.binary_op(|x, y| x | y),

            Op::GetVar(name) => {
                self.push(Cell::Val(*self.get_var(name).expect("variable not found")))
            }
            Op::SetVar(name) => {
                // TODO error propagation

                // dbg!("setvar start:", name);
                self.run().ok();
                // dbg!("setvar: end run");
                let val = self.pop_unchecked().unwrap_val();
                // dbg!("setvar", val);
                self.set_var(name, val);
            }
            Op::DeclVar(name) => {
                self.decl_var(name);
            }
            Op::FFI(ffi_fn) => {
                let mut params = DefaultStack::new();
                for param in ffi_fn.call_info() {
                    let top = self.top().ok_or(VMError::FixmeNotAnErrorExhausted)?;
                    match param {
                        Param::Normal => {
                            self.run().ok();
                            let pop = self.pop();
                            params.push(pop.ok_or(VMError::FixmeNotAnErrorExhausted)?);
                        }
                        Param::DynPacked => {
                            let param_len = (top.unwrap_raw() as usize).div_ceil(4) + 1;
                            // trench_trace!("param_len {}", param_len);
                            let stack_len = self.stack.len();

                            // trench_trace!("{stack_len} {param_len}");
                            let param_start = stack_len - param_len;

                            params.extend(self.stack[param_start..].iter().cloned());
                            self.stack.truncate(param_start);
                        }
                    }
                }
                // dbg!(ffi_fn, &params);
                let res = ffi_fn
                    .dispatch(&mut self.runtime, &params)
                    .and_then(|ffi_res| {
                        // TODO error propagation
                        self.return_stack.push(ffi_res);
                        // self.run(); // do something with the returned value
                        self.dump_state();
                        Ok(())
                    });

                return res;
            }
        }

        self.dump_state();
        // TODO
        Ok(())
    }

    pub fn add_func<P: AsRef<str>>(
        &mut self,
        name: impl AsRef<str>,
        params: &[P],
        stack: &[Cell<FFI>],
    ) {
        let mut fn_stack = Stack::new();
        fn_stack.extend(stack.iter().cloned());
        let name = name.as_ref();

        #[cfg(not(feature = "alloc"))]
        {
            if self.funcs.capacity() == 0 {
                panic!("out of function storage");
            }
        }
        self.funcs
            .insert(name.into(), FuncDef::new(params, fn_stack));
    }

    pub fn call_fn(&mut self, name: impl AsRef<str>) -> Result<(), VMError> {
        let name: VarString = name.as_ref().into();
        // drempels
        let func = self.funcs.get(&name).cloned();
        match func {
            Some(func) => {
                trench_debug!("calling {}", name);
                self.locals.push(VarStorage::new());

                self.return_addr = Some(self.stack.len());
                for param in &func.params {
                    self.stack.push(Op::SetVar(param.clone()).into());
                    self.stack.push(Op::DeclVar(param.clone()).into());
                    self.run().ok();
                }
                self.stack.push(Op::Nruter.into());
                self.stack.extend(func.stack.iter().cloned());

                let extra_verbose = !false;
                if extra_verbose {
                    self.dump_state();
                }
                let res = self.run();
                if extra_verbose {
                    self.dump_state();
                }
                trench_debug!("</{}>", name);
                self.locals.pop();
                res
            }
            None => Err(FFIError::FunctionNotFound.into()),
        }
    }

    pub fn decl_var(&mut self, name: impl AsRef<str>) {
        let name = name.as_ref().into();

        let storage = self.locals.last_mut().unwrap_or(&mut self.globals);
        #[cfg(not(feature = "alloc"))]
        {
            if storage.capacity() == 0 {
                panic!("variable space exhausted");
            }
        }
        storage.insert(name, None);
    }

    // JS semantics: assignment is always valid, if there's no local, it's a global
    fn var_assign_slot(&mut self, name: impl AsRef<str>) -> &mut Option<CellData> {
        let name = name.as_ref();

        if let Some(Entry::Occupied(local_entry)) = self
            .locals
            .last_mut()
            .map(|locals| locals.entry(name.into()))
        {
            return local_entry.into_mut();
        }

        let capacity = self.globals.capacity();
        match self.globals.entry(name.into()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(missing) => {
                #[cfg(not(feature = "alloc"))]
                {
                    if capacity == 0 {
                        panic!("global variable space exhausted");
                    }
                }
                missing.insert(None)
            }
        }
    }

    pub fn set_var(&mut self, name: impl AsRef<str>, val: CellData) {
        *self.var_assign_slot(name) = Some(val);
    }

    pub fn get_var(&self, name: impl AsRef<str>) -> Option<&CellData> {
        let name: &VarString = &name.as_ref().into();

        let res = match self.locals.last() {
            Some(locals) => locals.get(name).or(self.globals.get(name)),
            None => self.globals.get(name),
        };
        self.dump_state();
        res.expect("variable  not found").as_ref()
    }

    pub fn push(&mut self, i: Cell<FFI>) {
        trench_trace!("push {i:?}");
        #[cfg(not(feature = "alloc"))]
        {
            if self.stack.capacity() == 0 {
                err("stack overflow");
            }
        }
        self.stack.push(i);
    }

    pub fn pop(&mut self) -> Option<Cell<FFI>> {
        self.stack.pop()
    }

    pub fn pop_unchecked(&mut self) -> Cell<FFI> {
        let res = self.stack.pop();
        // trench_trace!("pop! {res:?}");
        if res.is_none() {
            err("stack not full enough");
        }
        res.unwrap()
    }

    pub fn push_return(&mut self, i: Cell<FFI>) {
        // trench_trace!("rpush {i:?}");
        #[cfg(not(feature = "alloc"))]
        {
            if self.return_stack.capacity() == 0 {
                err("return stack overflow");
            }
        }

        self.return_stack.push(i);
    }

    fn pop_return(&mut self) -> Cell<FFI> {
        let res = self.return_stack.pop();
        if res.is_none() {
            err("return stack underflow");
        }
        res.unwrap()
    }

    pub fn exit_fn(&mut self) {
        let ret = self.return_addr.take().expect("there's no return");
        self.stack.truncate(ret);
    }

    pub fn do_return(&mut self) {
        let top = self.pop_unchecked();

        #[cfg(not(feature = "alloc"))]
        {
            if self.return_stack.capacity() == 0 {
                err("return stack too full");
            }
        }
        self.return_stack.push(top);
    }

    pub fn top(&self) -> Option<&Cell<FFI>> {
        self.stack.last()
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        // run until exhausted
        // TODO very MEH architecture atm, a) cloned, b) the concept seems flawed
        // c) at the very least VMError::Exhausted/Done should not be an `Err`
        while let Some(Cell::Op(op)) = self.stack.last().cloned() {
            self.stack.pop();
            // trench_trace!("running {op:?}");
            self.dump_state();
            self.eval(&op);

            // trench_trace!("{op:?} done\n------------------------");
        }
        Err(VMError::FixmeNotAnErrorExhausted)
    }

    pub fn stack(&self) -> &[Cell<FFI>] {
        self.stack.as_ref()
    }

    pub fn runtime_mut(&mut self) -> &mut RT {
        &mut self.runtime
    }

    pub fn runtime(&self) -> &RT {
        &self.runtime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{forth::util::assert_similar, pixelblaze::ffi::PixelBlazeFFI};

    #[test]
    fn test_serde() -> anyhow::Result<()> {
        todo!();
        // let mut vm = vm();

        // vm.push(Cell::val(5));
        // vm.push(Cell::val(4));
        // vm.push(Cell::Op(Op::Add));
        // vm.push(Cell::val(10));
        // vm.push(Cell::Op(Op::Mul));

        // let ser: heapless::Vec<u8, 128> = postcard::to_vec(&vm)?;
        // let mut de: VM<ConsolePeripherals> = postcard::from_bytes(&ser)?;

        // de.run();
        // de.do_return();
        // assert_eq!(&[Cell::val(90)], &de.return_stack);
        // assert_eq!(&[], &de.stack);
        Ok(())
    }
}
