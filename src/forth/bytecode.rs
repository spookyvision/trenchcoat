use core::str::from_utf8;

use fixed::{types::extra::U8, FixedI32};
use serde::{Deserialize, Serialize};

pub type VarString = heapless::String<16>;
pub type Map<K, V, const N: usize> = heapless::FnvIndexMap<K, V, N>;
pub type CellData = FixedI32<U8>;
pub type VarStorage = Map<VarString, CellData, 8>;

pub type Stack<const N: usize> = heapless::Vec<Cell, N>;

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
        // TODO error propagation ("done" should not be an error...)
        vm.run().ok();
        let y = vm.pop().unwrap_val();
        vm.run().ok();
        let x = vm.pop().unwrap_val();

        vm.push(Cell::Val(op(x, y)));
    }
    fn eval(&self, vm: &mut VM) {
        // println!("----");
        // println!("eval {self:?}");
        match self {
            Op::Return => {
                vm.run().ok();
                let top = vm.pop();
                vm.return_stack.push(top).expect("return stack too full");
            }
            Op::Nruter => {
                // TODO: test
                let cell = vm.return_stack.pop().expect("return stack too empty");
                vm.stack.push(cell).expect("return stack too full");
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

            Op::GetVar(name) => vm.push(Cell::Val(
                *vm.get_var(name)
                    .expect(&format!("variable {name} not found")),
            )),
            Op::SetVar(name) => {
                // TODO error propagation
                vm.run().ok();
                let val = vm.pop().unwrap_val();
                vm.set_var(name, val);
            }
        }

        // vm.dump_state();
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Cell {
    Val(CellData),
    Op(Op),
}

impl Cell {
    // fn eval(&self, vm: &mut VM) -> Option<CellData> {
    //     println!("cell eval");
    //     match self {
    //         Cell::Val(val) => Some(*val),
    //         Cell::Op(op) => {
    //             op.eval(vm);
    //             None
    //             // vm.stack.last().unwrap().clone().eval(vm)
    //         }
    //     }
    // }
    // fn val(&self) -> Option<CellData> {
    //     match self {
    //         Cell::Val(val) => Some(*val),
    //         Cell::Op(_) => None,
    //     }
    // }
    fn unwrap_val(&self) -> CellData {
        match self {
            Cell::Val(val) => *val,
            Cell::Op(_) => panic!("tried to read value but found op"),
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct VM {
    stack: Stack<32>,
    return_stack: Stack<4>,
    globals: Map<VarString, CellData, 8>,
    locals: heapless::Vec<VarStorage, 4>,
    funcs: Map<VarString, Stack<64>, 4>,
}

#[derive(Debug, Clone, Copy)]
pub enum VMErr {
    Done,
    FunctionNotFound,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: heapless::Vec::new(),
            return_stack: heapless::Vec::new(),
            globals: Map::new(),
            locals: heapless::Vec::new(),
            funcs: Map::new(),
        }
    }
    pub fn dump_state(&self) {
        println!("stack: {:?}", self.stack);
        // println!("rstack: {:?}", self.return_stack);
        println!("globals: {:?}", self.globals);
        println!("locals: {:?}", self.locals);
        // println!("funcs: {:?}", self.funcs);
    }

    pub fn add_func(&mut self, name: impl AsRef<str>, stack: &[Cell]) {
        let mut our_stack = Stack::new();
        our_stack.extend(stack.iter().cloned());
        let name = name.as_ref();
        dbg!(name);
        self.funcs.insert(name.into(), our_stack).expect("oh no");
    }

    pub fn call_fn(&mut self, name: impl AsRef<str>) -> Result<(), VMErr> {
        let name = name.as_ref();
        // drempels
        let ops = self.funcs.get(&name.into());
        match ops {
            Some(ops) => {
                self.locals.push(VarStorage::new());
                self.stack.extend(ops.iter().cloned());
                println!("calling {name}");
                self.dump_state();
                let res = self.run();
                self.dump_state();
                println!("</{name}>");
                self.locals.pop();
                res
            }
            None => Err(VMErr::FunctionNotFound),
        }
    }

    // TODO null/undefined?
    pub fn set_var(&mut self, name: impl AsRef<str>, val: CellData) {
        let name = name.as_ref().into();

        // TODO strictly speaking the caller should decide if it's a global
        let context = self.locals.last_mut().unwrap_or(&mut self.globals);
        context.insert(name, val).expect("variable space exhausted");
    }

    pub fn get_var(&self, name: impl AsRef<str>) -> Option<&CellData> {
        let name = name.as_ref().into();
        let vars = self.locals.last().unwrap_or(&self.globals);
        vars.get(&name)
    }

    pub fn push(&mut self, i: Cell) {
        // println!("push {i:?}");
        if let Err(e) = self.stack.push(i) {
            err("stack too full");
        }
    }

    fn pop(&mut self) -> Cell {
        let res = self.stack.pop();
        // println!("pop! {res:?}");
        if res.is_none() {
            err("stack not full enough");
        }
        res.unwrap()
    }

    pub fn push_return(&mut self, i: Cell) {
        // println!("rpush {i:?}");
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
        while let Some(Cell::Op(op)) = self.stack.last().cloned() {
            self.stack.pop();
            op.eval(self);
            // self.dump_state();

            // println!("{op:?} done\n------------------------");
        }
        Err(VMErr::Done)
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
        self.push(Cell::Val(CellData::from_num(valid_bytes_len)));
    }

    pub fn get_str(&mut self) -> &str {
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

        // TODO bytemuck or whatever
        let string_slice = unsafe {
            core::slice::from_raw_parts(almost_string_stack.as_ptr() as *const u8, string_bytes_len)
        };

        stack.truncate(string_start);
        dbg!(string_slice, string_start, stack_items_len);

        from_utf8(string_slice).unwrap_or("<err>")
    }

    pub fn stack(&self) -> &[Cell] {
        self.stack.as_ref()
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
    assert_eq!(&[Cell::Val((90).into())], &de.return_stack);
    assert_eq!(&[], &de.stack);
    Ok(())
}
