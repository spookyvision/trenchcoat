use core::{fmt::Debug, marker::PhantomData, str::from_utf8};

use fixed::{traits::ToFixed, types::extra::U16, FixedI32};

#[cfg(feature = "alloc")]
extern crate alloc;

use serde::{Deserialize, Serialize};

#[cfg(not(feature = "alloc"))]
pub(crate) mod types {
    use super::Cell;

    pub type VarString = heapless::String<16>;
    pub type Map<K, V, const N: usize> = heapless::FnvIndexMap<K, V, N>;

    pub type Stack<FFI, const N: usize> = heapless::Vec<Cell<FFI>, N>;

    pub type VMVec<T, const N: usize> = heapless::Vec<T, N>;
}

#[cfg(all(feature = "alloc", not(feature = "use-std")))]
use alloc::collections::btree_map::Entry;
#[cfg(feature = "use-std")]
use std::collections::hash_map::Entry;

#[cfg(not(feature = "alloc"))]
use heapless::Entry;

#[cfg(feature = "alloc")]

pub(crate) mod types {

    use super::Cell;

    pub type VarString = alloc::string::String;
    #[cfg(feature = "use-std")]
    pub type Map<K, V, const N: usize> = std::collections::HashMap<K, V>;
    #[cfg(not(feature = "use-std"))]
    pub type Map<K, V, const N: usize> = alloc::collections::btree_map::BTreeMap<K, V>;

    pub type Stack<FFI, const N: usize> = alloc::vec::Vec<Cell<FFI>>;
    pub type VMVec<T, const N: usize> = alloc::vec::Vec<T>;
}

pub use types::*;

use super::util::StackSlice;

pub type DefaultStack<FFI> = Stack<FFI, 64>;

// TODO pixelblaze uses <16,16> but that's not the best general range
// -> definitely feature gate this to at least have <24,8>
// -> bite the `f32` bullet?
// (strict JS compliance would need `f64`, also "bitwise operations will convert it to a 32 bit integer."
// https://www.ecma-international.org/publications/files/ECMA-ST/Ecma-262.pdf

pub type CellData = FixedI32<U16>;

// TODO why Option... presumably for null? If so, make a better API
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
    #[cfg_attr(feature = "use-std", error("Variable not found"))]
    VarNotFound,
    #[cfg_attr(feature = "use-std", error("VM vanished"))]
    Vanished,
    #[cfg_attr(feature = "use-std", error("Val"))]
    Val(#[cfg_attr(feature = "use-std", from)] ValError),
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
    EqEq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    If,
    Then,
    Else,
    CallDyn,
    // TODO can we get rid of these strings?
    // TODO can we optimize Call in particular?
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

#[allow(unused)]
trait BoolExt {
    fn to_fixed(&self) -> CellData;
}

impl BoolExt for bool {
    fn to_fixed(&self) -> CellData {
        match self {
            true => CellData::from_num(1),
            false => CellData::from_num(0),
        }
    }
}

// TODO use Option<Cell> instead of `Cell::Null`?
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Cell<FFI> {
    Val(CellData),
    Raw(i32),
    Op(Op<FFI>),
    Null,
}

// no-specialization fuckery workaround
pub(crate) trait ToNull<FFI> {
    fn to_null(&self) -> Cell<FFI>;
}

impl<T> ToNull<T> for () {
    fn to_null(&self) -> Cell<T> {
        Cell::Null
    }
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

#[cfg_attr(feature = "use-std", derive(thiserror::Error))]
#[derive(Debug, Serialize, Deserialize)]
pub enum ValError {
    #[cfg_attr(feature = "use-std", error("tried to read value but found op"))]
    Op,
    #[cfg_attr(feature = "use-std", error("tried to read raw"))]
    Raw,
    #[cfg_attr(feature = "use-std", error("tried to read null"))]
    Null,
}

impl<FFI> Cell<FFI> {
    pub(crate) fn val(num: impl ToFixed) -> Self {
        Self::Val(num.to_fixed())
    }
    pub(crate) fn checked_val(&self) -> Result<CellData, ValError> {
        match self {
            Cell::Val(val) => Ok(*val),
            Cell::Op(_) => Err(ValError::Op),
            Cell::Raw(_) => Err(ValError::Raw),
            Cell::Null => Err(ValError::Null),
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

    fn binary_op(&mut self, op: BinOp) -> Result<(), VMError> {
        // TODO error propagation ("Exhausted" should not be an error...)
        // trench_debug!("\n\n\n\n---bop\n");
        self.dump_state();
        self.run()?;
        self.dump_state();
        let y = self.pop()?.checked_val()?;
        self.run()?;
        let x = self.pop()?.checked_val()?;

        self.push(Cell::Val(op(x, y)));
        Ok(())
    }

    fn eval(&mut self, op: &Op<FFI>) -> Result<(), VMError> {
        // trench_debug!("----");
        // trench_debug!("eval {self:?}");
        match op {
            Op::ExitFn => {
                self.exit_fn();
            }
            Op::PopRet => {
                self.pop_return()?;
            }
            Op::Return => {
                self.do_return();
            }
            // TODO: test
            Op::Nruter => {
                let cell = self.pop_return()?;
                if self.stack.capacity() == 0 {
                    return Err(VMError::Overflow);
                }
                self.stack.push(cell);
            }
            Op::CallDyn => {
                // TODO pasta
                let top = self.top().ok_or(VMError::Underflow)?;
                let name_len = (top.unwrap_raw() as usize).div_ceil(4) + 1;
                let stack_len = self.stack.len();

                let name_start = stack_len - name_len;

                let v: heapless::Vec<u8, 32> = StackSlice(&self.stack[name_start..])
                    .try_into()
                    .map_err(|_| VMError::Malformed)?;
                let name = from_utf8(&v).map_err(|_| VMError::Malformed)?;
                self.stack.truncate(name_start);
                trench_debug!("call_dyn {name}");
            }
            Op::Call(name) => {
                if let Err(e) = self.call_fn(name) {
                    trench_debug!("Call {e:?}");
                }
            }
            Op::If => return Err(VMError::Malformed),
            Op::Then => {
                let mut if_idx = None;
                let mut else_idx = None;
                for (idx, elem) in self.stack.iter().enumerate().rev() {
                    match elem {
                        Cell::Op(op) => {
                            if *op == Op::If {
                                if_idx = Some(idx);
                                break;
                            }
                            if *op == Op::Else {
                                else_idx = Some(idx);
                            }
                        }
                        _ => {
                            continue;
                        }
                    }
                }

                // TODO horrible, plz refactor a la
                // https://github.com/dewaka/forth-rs/blob/master/src/forth/inter.rs#L111
                match if_idx {
                    None | Some(0) => return Err(VMError::Malformed),
                    Some(if_idx) => {
                        // set aside
                        let mut else_part: Option<Vec<_>> = None;
                        let if_part: Vec<_> = match else_idx {
                            Some(else_idx) => {
                                else_part = Some(self.stack.drain(else_idx..).collect());
                                self.stack.drain(if_idx..else_idx).collect()
                            }
                            None => self.stack.drain(if_idx..).collect(),
                        };

                        // check condition on top of stack
                        self.run()?;
                        match self.top() {
                            Some(Cell::Val(cond)) => {
                                // TODO better bool handling?
                                if *cond == CellData::from_num(1) {
                                    self.stack.extend(if_part);
                                } else {
                                    if else_part.is_some() {
                                        self.stack.extend(else_part.unwrap());
                                    }
                                }
                            }
                            _ => return Err(VMError::Malformed),
                        }
                    }
                }
            }
            Op::Else => return Err(VMError::Malformed),

            Op::EqEq => self.binary_op(|x, y| (x == y).to_fixed())?,
            Op::NotEq => self.binary_op(|x, y| (x != y).to_fixed())?,
            Op::Lt => self.binary_op(|x, y| (x < y).to_fixed())?,
            Op::LtEq => self.binary_op(|x, y| (x <= y).to_fixed())?,
            Op::Gt => self.binary_op(|x, y| (x > y).to_fixed())?,
            Op::GtEq => self.binary_op(|x, y| (x >= y).to_fixed())?,
            Op::Add => self.binary_op(|x, y| x + y)?,
            Op::Sub => self.binary_op(|x, y| x - y)?,
            Op::Mul => self.binary_op(|x, y| x * y)?,
            Op::Div => self.binary_op(|x, y| x / y)?,
            Op::Mod => self.binary_op(|x, y| x % y)?,
            Op::And => self.binary_op(|x, y| x & y)?,
            Op::Or => self.binary_op(|x, y| x | y)?,

            Op::GetVar(name) => {
                let var_res = self.get_var(name)?;
                let var = var_res.expect("variable not found/is null (TODO fixme)");
                self.push(Cell::Val(var))
            }
            Op::SetVar(name) => {
                // TODO error propagation

                // dbg!("setvar start:", name);
                self.run()?;
                // dbg!("setvar: end run");
                let val = self.pop()?.checked_val()?;
                // dbg!("setvar", val);
                self.set_var(name, val);
            }
            Op::DeclVar(name) => {
                self.decl_var(name);
            }
            Op::FFI(ffi_fn) => {
                let mut params = DefaultStack::new();
                for param in ffi_fn.call_info() {
                    let top = self.top().ok_or(VMError::Underflow)?;
                    match param {
                        Param::Normal => {
                            self.run()?;
                            let cell = self.pop()?;
                            params.push(cell);
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
                    self.run()?;
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

        match self.globals.entry(name.into()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(missing) => {
                #[cfg(not(feature = "alloc"))]
                {
                    missing
                        .insert(None)
                        .expect("global variable space exhausted")
                }
                #[cfg(feature = "alloc")]
                {
                    missing.insert(None)
                }
            }
        }
    }

    pub fn set_var(&mut self, name: impl AsRef<str>, val: CellData) {
        *self.var_assign_slot(name) = Some(val);
    }

    pub fn get_var(&self, name: impl AsRef<str>) -> Result<&Option<CellData>, VMError> {
        let name: &VarString = &name.as_ref().into();

        let res = match self.locals.last() {
            Some(locals) => locals.get(name).or(self.globals.get(name)),
            None => self.globals.get(name),
        };
        self.dump_state();
        res.ok_or(VMError::VarNotFound)
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

    pub fn pop(&mut self) -> Result<Cell<FFI>, VMError> {
        self.stack.pop().ok_or(VMError::Underflow)
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

    fn pop_return(&mut self) -> Result<Cell<FFI>, VMError> {
        self.return_stack.pop().ok_or(VMError::Underflow)
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
            self.eval(&op)?;

            // trench_trace!("{op:?} done\n------------------------");
        }
        Ok(())
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

    pub fn funcs(&self) -> &DefaultFuncDef<FFI> {
        &self.funcs
    }

    pub fn globals(&self) -> &VarStorage {
        &self.globals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        forth::{
            compiler::{compile, Flavor, Source},
            util::test::assert_similar,
        },
        pixelblaze::{ffi::PixelBlazeFFI, runtime::ConsoleRuntime},
        vanillajs::runtime::VanillaJSFFI,
    };

    #[test]
    fn test_serde() -> anyhow::Result<()> {
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

    #[test]
    fn test_if() -> anyhow::Result<()> {
        let source = r#"
    if (1 < 0) {
        x = 1
    } else {
        x = 2
    }
    "#;

        let mut bytecode = compile(Source::String(source), Flavor::VanillaJS)?;
        let mut de: VM<VanillaJSFFI, ConsoleRuntime> = postcard::from_bytes_cobs(&mut bytecode)?;
        de.run();
        let x = *de.get_var("x")?;
        assert_eq!(x, Some(CellData::from_num(2)));
        Ok(())
    }
}
