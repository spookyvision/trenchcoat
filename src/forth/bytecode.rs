use core::str::from_utf8;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub type VarString = heapless::String<8>;
pub type Map<K, V, const N: usize> = heapless::FnvIndexMap<K, V, N>;
pub type CellData = i32;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Op {
    Return, // data stack -> return stack
    Nruter, // return stack -> data stack
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Pop,
    FFI(FFI),
    Call(VarString),
    SetVar(VarString),
    GetVar(VarString),
}

type BinOp = fn(CellData, CellData) -> CellData;

fn err(s: &str) {
    panic!("ERR: {s}")
}
impl Op {
    fn binary_op(vm: &mut VM, op: BinOp) {
        let x = vm.pop().eval(vm);
        let y = vm.pop().eval(vm);
        vm.push(Cell::Val(op(x, y)));
        println!("-- end bop --");
    }
    fn eval(&self, vm: &mut VM) {
        println!("----");
        println!("eval {self:?}");
        match self {
            Op::Return => {
                let top = vm.pop();
                vm.return_stack.push(top).expect("return stack too full");
            }
            Op::Nruter => {
                // TODO: test
                let cell = vm.pop();
                vm.return_stack.push(cell).expect("return stack too full");
            }
            Op::Call(name) => {
                vm.call_fn(name);
            }
            Op::Add => Self::binary_op(vm, |x, y| x + y),
            Op::Sub => Self::binary_op(vm, |x, y| x - y),
            Op::Mul => Self::binary_op(vm, |x, y| x * y),
            Op::Div => Self::binary_op(vm, |x, y| x / y),
            Op::Mod => Self::binary_op(vm, |x, y| x % y),
            Op::And => Self::binary_op(vm, |x, y| x & y),
            Op::Or => Self::binary_op(vm, |x, y| x | y),
            Op::Pop => {
                let _ = vm.pop();
            }
            Op::FFI(ffi_fn) => match ffi_fn {
                FFI::ConsoleLog1 => {
                    let str = vm.get_str();
                    console_log(str);
                }
            },

            Op::GetVar(name) => vm.push(Cell::Val(*vm.get_var(name).expect("variable not found"))),
            Op::SetVar(name) => {
                vm.run().expect("boom");
                let val = vm.pop().unwrap_val();
                vm.set_var(name, val);
            }
        }

        vm.dump_state();
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Cell {
    Val(CellData),
    Op(Op),
}

impl Cell {
    fn eval(&self, vm: &mut VM) -> CellData {
        println!("cell eval");
        match self {
            Cell::Val(val) => *val,
            Cell::Op(op) => {
                op.eval(vm);
                vm.stack.last().unwrap().clone().eval(vm)
            }
        }
    }
    fn val(&self) -> Option<CellData> {
        match self {
            Cell::Val(val) => Some(*val),
            Cell::Op(_) => None,
        }
    }
    fn unwrap_val(&self) -> CellData {
        match self {
            Cell::Val(val) => *val,
            Cell::Op(_) => panic!("tried to read value but found op"),
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct VM {
    stack: heapless::Vec<Cell, 32>,
    return_stack: heapless::Vec<Cell, 4>,
    globals: heapless::FnvIndexMap<VarString, CellData, 8>,
    locals: Option<heapless::FnvIndexMap<VarString, CellData, 8>>,
    funcs: heapless::FnvIndexMap<VarString, heapless::Vec<Cell, 64>, 4>,
}

#[derive(Debug, Clone, Copy)]
pub enum VMErr {
    Done,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: heapless::Vec::new(),
            return_stack: heapless::Vec::new(),
            globals: Map::new(),
            locals: None,
            funcs: Map::new(),
        }
    }
    pub fn dump_state(&self) {
        println!("stack: {:?}", self.stack);
        println!("rstack: {:?}", self.return_stack);
        println!("vars: {:?}", self.globals);
    }

    // TODO null/undefined?
    pub fn set_var(&mut self, name: impl AsRef<str>, val: CellData) {
        let name = name.as_ref().into();

        // TODO strictly speaking the caller should decide if it's a global
        let context = self.locals.as_mut().unwrap_or(&mut self.globals);
        context.insert(name, val).expect("variable space exhausted");
    }

    pub fn call_fn(&mut self, name: impl AsRef<str>) {
        let mut func = self
            .funcs
            .get(&name.as_ref().into())
            .expect("function {name} not found")
            .clone();

        // TODO goes boom with call stack > <
        self.locals = Some(Map::new());

        while let Some(cell) = func.pop() {
            cell.eval(self);
        }
        self.locals = None;
    }

    pub fn get_var(&self, name: impl AsRef<str>) -> Option<&CellData> {
        let name = name.as_ref().into();
        self.globals.get(&name)
    }

    pub fn push(&mut self, i: Cell) {
        println!("push {i:?}");
        if let Err(e) = self.stack.push(i) {
            err("stack too full");
        }
    }

    fn pop(&mut self) -> Cell {
        let res = self.stack.pop();
        println!("pop! {res:?}");
        if res.is_none() {
            err("stack not full enough");
        }
        res.unwrap()
    }

    pub fn push_return(&mut self, i: Cell) {
        println!("rpush {i:?}");
        if let Err(e) = self.return_stack.push(i) {
            err("return stack too full");
        }
    }

    fn pop_return(&mut self) -> Cell {
        let res = self.return_stack.pop();
        if res.is_none() {
            err("return stack not full enough");
        }
        res.unwrap()
    }

    pub fn run(&mut self) -> Result<(), VMErr> {
        // TODO meh, would rather not clone
        let last = self.stack.last().cloned();
        if let Some(Cell::Op(op)) = last {
            self.stack.pop();
            println!("run {op:?}");
            println!("---");
            op.eval(self);
            self.dump_state();
            println!("---");
            Ok(())
        } else {
            Err(VMErr::Done)
        }
    }

    pub fn push_str(&mut self, s: impl AsRef<str>) {
        let s = s.as_ref();
        println!("push str {s:?}");
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
        self.push(Cell::Val(valid_bytes_len as CellData));
    }

    pub fn get_str(&mut self) -> &str {
        let stack = &mut self.stack;
        let string_bytes_len = stack
            .pop()
            .expect("could not read string length: stack is empty")
            .unwrap_val() as usize;
        let stack_items_len = 1 + (string_bytes_len >> 2);
        let stack_slice = stack.as_slice();
        let string_start = stack.len() - stack_items_len;
        let almost_string_stack = &stack_slice[string_start..][..stack_items_len];

        // TODO bytemuck or whatever
        let string_slice = unsafe {
            core::slice::from_raw_parts(almost_string_stack.as_ptr() as *const u8, string_bytes_len)
        };

        stack.truncate(string_start);

        from_utf8(string_slice).unwrap_or("<err>")
    }
}
#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum FFI {
    ConsoleLog1,
}

fn console_log(s: &str) {
    println!("hullo from rust: >>>{s}<<<")
}

#[test]
fn test_ffi() -> anyhow::Result<()> {
    let mut vm = VM::new();
    vm.push_str("⭐hello, vm!⭐");
    vm.push(Cell::Op(Op::FFI(FFI::ConsoleLog1)));
    Ok(())
}
#[test]
fn test_ret() -> anyhow::Result<()> {
    let mut vm = VM::new();

    vm.push(Cell::Val(5));
    vm.push(Cell::Val(4));
    vm.push(Cell::Op(Op::Add));
    vm.push(Cell::Val(10));
    vm.push(Cell::Op(Op::Mul));
    vm.push(Cell::Op(Op::Return));

    let ser: heapless::Vec<u8, 128> = postcard::to_vec(&vm)?;
    let mut de: VM = postcard::from_bytes(&ser)?;

    println!("vm start");
    de.dump_state();
    while let Ok(_) = de.run() {}
    assert_eq!(&[Cell::Val(90)], &de.return_stack);
    assert_eq!(&[], &de.stack);
    Ok(())
}
