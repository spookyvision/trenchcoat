use core::str::from_utf8;

use fixed::{traits::ToFixed, types::extra::U8, FixedI32};
use log::trace;
use serde::{Deserialize, Serialize};

pub trait FFI<VM> {
    fn dispatch(&self, vm: &mut VM);
}

pub type VarString = heapless::String<16>;
pub type Map<K, V, const N: usize> = heapless::FnvIndexMap<K, V, N>;
// TODO pixelblaze uses <16,16> but that's not the best general range
// -> add flavors
pub type CellData = FixedI32<U8>;
pub type VarStorage = Map<VarString, Option<CellData>, 32>;

pub type Stack<FFI, const N: usize> = heapless::Vec<Cell<FFI>, N>;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Op<FFI> {
    Return, // data stack -> return stack
    Nruter, // return stack -> data stack
    ExitFn,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Pop,
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

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Cell<FFI> {
    Val(CellData),
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
            Cell::Null => panic!("tried to read null"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FuncDef<FFI> {
    stack: Stack<FFI, 64>,
    params: heapless::Vec<VarString, 4>,
}

impl<FFI> FuncDef<FFI> {
    fn new<P: AsRef<str>>(stack: Stack<FFI, 64>, params: &[P]) -> Self {
        let mut our_params = heapless::Vec::new();
        for param in params {
            our_params.push(param.as_ref().into());
        }
        Self {
            stack,
            params: our_params,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VMErr {
    Done,
    FunctionNotFound,
}
#[derive(Serialize, Deserialize)]
pub struct VM<FFI_GEN, RT> {
    stack: Stack<FFI_GEN, 64>,
    return_stack: Stack<FFI_GEN, 4>,
    return_addr: Option<usize>,
    globals: VarStorage,
    locals: heapless::Vec<VarStorage, 8>,
    funcs: Map<VarString, FuncDef<FFI_GEN>, 8>,
    #[serde(skip)]
    runtime: RT,
}

impl<FFI_GEN, RT> VM<FFI_GEN, RT>
where
    FFI_GEN: core::fmt::Debug + Clone + FFI<Self>,
{
    pub fn new(runtime: RT) -> Self {
        Self {
            stack: heapless::Vec::new(),
            return_stack: heapless::Vec::new(),
            return_addr: None,
            globals: Map::new(),
            locals: heapless::Vec::new(),
            funcs: Map::new(),
            runtime,
        }
    }

    pub fn dump_state(&self) {
        log::debug!("stack: {:?}", self.stack);
        log::debug!("rstack: {:?}", self.return_stack);
        log::debug!("globals: {:?}", self.globals);
        log::debug!("locals: {:?}", self.locals);
        let debug_funcs = true;
        if debug_funcs {
            for (name, def) in &self.funcs {
                log::debug!("F {name} => {def:?}")
            }
        }
    }

    fn binary_op(&mut self, op: BinOp) {
        // TODO error propagation ("done" should not be an error...)
        self.run().ok();
        let y = self.pop().unwrap_val();
        self.run().ok();
        let x = self.pop().unwrap_val();

        self.push(Cell::Val(op(x, y)));
    }

    fn eval(&mut self, op: &Op<FFI_GEN>) {
        // println!("----");
        // println!("eval {self:?}");
        match op {
            Op::ExitFn => {
                self.exit_fn();
            }
            Op::Return => {
                self.do_return();
            }
            Op::Nruter => {
                // TODO: test
                let cell = self.return_stack.pop().expect("return stack too empty");
                self.stack.push(cell).expect("return stack too full");
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
            Op::Pop => {
                let _ = self.pop();
            }

            Op::GetVar(name) => self.push(Cell::Val(
                *self
                    .get_var(name)
                    .expect(&format!("variable {name} not found")),
            )),
            Op::SetVar(name) => {
                // TODO error propagation

                self.run().ok();
                let val = self.pop().unwrap_val();
                self.set_var(name, val);
            }
            Op::DeclVar(name) => {
                self.decl_var(name);
            }
            Op::FFI(ffi_fn) => ffi_fn.dispatch(&mut self),
        }

        // vm.dump_state();
    }

    pub fn add_func<P: AsRef<str>>(
        &mut self,
        name: impl AsRef<str>,
        params: &[P],
        stack: &[Cell<FFI_GEN>],
    ) {
        let mut fn_stack = Stack::new();
        fn_stack.extend(stack.iter().cloned());
        let name = name.as_ref();
        dbg!(name);
        self.funcs
            .insert(name.into(), FuncDef::new(fn_stack, params))
            .expect("oh no");
    }

    pub fn call_fn(&mut self, name: impl AsRef<str>) -> Result<(), VMErr> {
        let name = name.as_ref();
        // drempels
        let func = self.funcs.get(&name.into()).cloned();
        match func {
            Some(func) => {
                println!("calling {name}");
                self.locals.push(VarStorage::new());

                self.return_addr = Some(self.stack.len());
                for param in &func.params {
                    self.stack.push(Op::SetVar(param.clone()).into());
                    self.stack.push(Op::DeclVar(param.clone()).into());
                    self.run().ok();
                }
                self.stack.push(Op::Nruter.into());
                self.stack.extend(func.stack.iter().cloned());

                let extra_verbose = false;
                if extra_verbose {
                    self.dump_state();
                }
                let res = self.run();
                if extra_verbose {
                    self.dump_state();
                }
                println!("</{name}>");
                self.locals.pop();
                res
            }
            None => Err(VMErr::FunctionNotFound),
        }
    }

    pub fn decl_var(&mut self, name: impl AsRef<str>) {
        let name = name.as_ref().into();

        let storage = self.locals.last_mut().unwrap_or(&mut self.globals);
        storage
            .insert(name, None)
            .expect("variable space exhausted");
    }

    // JS semantics: assignment is always valid, if there's no local, it's a global
    fn var_assign_slot(&mut self, name: impl AsRef<str>) -> &mut Option<CellData> {
        let name = name.as_ref();

        if let Some(heapless::Entry::Occupied(local_entry)) = self
            .locals
            .last_mut()
            .map(|locals| locals.entry(name.into()))
        {
            return local_entry.into_mut();
        }

        match self.globals.entry(name.into()) {
            heapless::Entry::Occupied(entry) => entry.into_mut(),
            heapless::Entry::Vacant(missing) => missing
                .insert(None)
                .expect("global variable space exhausted"),
        }
    }

    pub fn set_var(&mut self, name: impl AsRef<str>, val: CellData) {
        *self.var_assign_slot(name) = Some(val);
    }

    pub fn get_var(&self, name: impl AsRef<str>) -> Option<&CellData> {
        let name = &name.as_ref().into();

        let res = match self.locals.last() {
            Some(locals) => locals.get(name).or(self.globals.get(name)),
            None => self.globals.get(name),
        };
        self.dump_state();
        res.expect(&format!("variable {name} not found")).as_ref()
    }

    pub fn push(&mut self, i: Cell<FFI_GEN>) {
        // println!("push {i:?}");
        if let Err(e) = self.stack.push(i) {
            err("stack too full");
        }
    }

    pub fn pop(&mut self) -> Cell<FFI_GEN> {
        let res = self.stack.pop();
        // println!("pop! {res:?}");
        if res.is_none() {
            err("stack not full enough");
        }
        res.unwrap()
    }

    pub fn push_return(&mut self, i: Cell<FFI_GEN>) {
        // println!("rpush {i:?}");
        if let Err(e) = self.return_stack.push(i) {
            err("return stack too full");
        }
    }

    fn pop_return(&mut self) -> Cell<FFI_GEN> {
        let res = self.return_stack.pop();
        if res.is_none() {
            err("return stack not full enough");
        }
        res.unwrap()
    }

    pub fn exit_fn(&mut self) {
        let ret = self.return_addr.take().expect("there's no return");
        self.stack.truncate(ret);
    }

    pub fn do_return(&mut self) {
        let top = self.pop();
        self.return_stack.push(top).expect("return stack too full");
    }

    pub fn run(&mut self) -> Result<(), VMErr> {
        while let Some(Cell::Op(op)) = self.stack.pop() {
            trace!("running {op:?}");
            self.dump_state();
            self.eval(&op);

            trace!("{op:?} done\n------------------------");
        }
        Err(VMErr::Done)
    }

    pub fn push_str(&mut self, s: impl AsRef<str>) {
        let s = s.as_ref();
        log::debug!("push str {s:?}");
        let bytes = s.as_bytes();
        let valid_bytes_len = bytes.len();
        let chonky_boytes = bytes.chunks_exact(4);
        let remainder = chonky_boytes.remainder();
        for item in chonky_boytes {
            let i = CellData::from_le_bytes(<[u8; 4]>::try_from(item).expect("unreachable"));
            self.push(Cell::Val(i));
        }
        let mut remaining_chonk = [0u8; 4];
        remaining_chonk[0..remainder.len()].copy_from_slice(remainder);
        self.push(Cell::Val(CellData::from_le_bytes(remaining_chonk)));
        self.push(Cell::Val(CellData::from_num(valid_bytes_len)));
    }

    pub fn get_str(&mut self) -> impl AsRef<str> {
        let stack = &mut self.stack;
        let string_bytes_len: usize = stack
            .pop()
            .expect("could not read string length: stack is empty")
            .unwrap_val()
            .to_num();
        let stack_items_len = (string_bytes_len >> 2) + 1;
        let stack_slice = stack.as_slice();
        let string_start = stack.len() - stack_items_len;
        let almost_string_stack = &stack_slice[string_start..][..stack_items_len];

        let res = &mut [0u8; 32];

        for (i, packed_bytes) in almost_string_stack
            .iter()
            .map(|elem| elem.unwrap_val())
            .enumerate()
        {
            res[i * 4..][..4].copy_from_slice(&packed_bytes.to_le_bytes());
        }

        stack.truncate(string_start);
        let stack_str = from_utf8(&res[..string_bytes_len]).unwrap_or("<err>");
        let res: heapless::String<32> = stack_str.into();
        res
    }

    pub fn stack(&self) -> &[Cell<FFI_GEN>] {
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
    use crate::{forth::util::assert_similar, pixelblaze::funcs::PixelBlazeFFI};

    #[test]
    fn test_str() -> anyhow::Result<()> {
        let mut vm = VM::new(ConsolePeripherals);
        let s = "⭐hello, vm!⭐";
        vm.push_str(s);
        assert_eq!(vm.get_str().as_ref(), s);

        Ok(())
    }

    #[test]
    fn test_serde() -> anyhow::Result<()> {
        let mut vm = vm();

        vm.push(Cell::val(5));
        vm.push(Cell::val(4));
        vm.push(Cell::Op(Op::Add));
        vm.push(Cell::val(10));
        vm.push(Cell::Op(Op::Mul));

        let ser: heapless::Vec<u8, 128> = postcard::to_vec(&vm)?;
        let mut de: VM<ConsolePeripherals> = postcard::from_bytes(&ser)?;

        de.run();
        de.do_return();
        assert_eq!(&[Cell::val(90)], &de.return_stack);
        assert_eq!(&[], &de.stack);
        Ok(())
    }
}
