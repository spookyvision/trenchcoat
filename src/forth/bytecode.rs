use core::str::from_utf8;

use axum::body::HttpBody;
use serde::{Deserialize, Serialize};

pub type VarString = heapless::String<8>;

#[derive(Serialize, Deserialize, Debug)]
pub enum Ops {
    Data(i32),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Pop,
    FFI(FFI),
    SetVar(VarString),
    GetVar(VarString),
}

type BinOp = fn(i32, i32) -> i32;

/*

pub fn modulus(vm: &mut VM) {
    binary_op("mod", |x, y| y % x, vm)
}

pub fn and(vm: &mut VM) {
    binary_op("and", |x, y| y & x, vm)
}

pub fn or(vm: &mut VM) {
    binary_op("or", |x, y| y | x, vm)
}
*/
fn err(s: &str) {
    panic!("ERR: {s}")
}
impl Ops {
    fn binary_op(vm: &mut VM, op: BinOp) {
        let x = vm.pop();
        let y = vm.pop();
        vm.push(op(x, y))
    }
    fn eval(&self, vm: &mut VM) {
        match self {
            Ops::Data(n) => vm.push(*n),
            Ops::Add => Self::binary_op(vm, |x, y| x + y),
            Ops::Sub => Self::binary_op(vm, |x, y| x - y),
            Ops::Mul => Self::binary_op(vm, |x, y| x * y),
            Ops::Div => Self::binary_op(vm, |x, y| x / y),
            Ops::Mod => Self::binary_op(vm, |x, y| x % y),
            Ops::And => Self::binary_op(vm, |x, y| x & y),
            Ops::Or => Self::binary_op(vm, |x, y| x | y),
            Ops::Pop => {
                let _ = vm.pop();
            }
            Ops::FFI(ffi_fn) => match ffi_fn {
                FFI::ConsoleLog1 => {
                    let str = vm.get_str();
                    console_log(str);
                }
            },
            Ops::GetVar(name) => vm.push(*vm.get_var(name).expect("variable not found")),
            Ops::SetVar(name) => {
                let val = vm.pop();
                vm.set_var(name, val);
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VM {
    data: heapless::Vec<i32, 128>,
    ops: heapless::Vec<Ops, 128>,
    vars: heapless::FnvIndexMap<VarString, i32, 8>,
}

pub enum VMErr {
    Done,
}

impl VM {
    pub fn new() -> Self {
        Self {
            data: heapless::Vec::new(),
            ops: heapless::Vec::new(),
            vars: heapless::FnvIndexMap::new(),
        }
    }
    pub fn dump_state(&self) {
        println!("data: {:?}", self.data);
        println!("ops: {:?}", self.ops);
        println!("vars: {:?}", self.vars);
    }

    // TODO null/undefined?
    pub fn set_var(&mut self, name: impl AsRef<str>, val: i32) {
        let name = name.as_ref().into();
        self.vars
            .insert(name, val)
            .expect("variable space exhausted");
    }

    pub fn get_var(&self, name: impl AsRef<str>) -> Option<&i32> {
        let name = name.as_ref().into();
        self.vars.get(&name)
    }

    pub fn push(&mut self, i: i32) {
        println!("push data {i}");
        if let Err(e) = self.data.push(i) {
            err("data stack too full");
        }
    }

    fn pop(&mut self) -> i32 {
        let res = self.data.pop();
        if res.is_none() {
            err("data stack not full enough");
        }
        res.unwrap()
    }

    pub fn step(&mut self) -> Result<(), VMErr> {
        if let Some(op) = self.ops.pop() {
            println!("exec {op:?}");
            op.eval(self);
            Ok(())
        } else {
            Err(VMErr::Done)
        }
    }
    pub fn push_op(&mut self, op: Ops) {
        println!("push op {op:?}");
        if let Err(e) = self.ops.push(op) {
            err("op stack too full");
        }
    }
    pub fn push_str(&mut self, s: impl AsRef<str>) {
        let data_stack = &mut self.data;
        let s = s.as_ref();
        println!("push str {s:?}");
        let bytes = s.as_bytes();
        let valid_bytes_len = bytes.len();
        let chonky_boytes = bytes.chunks_exact(4);
        let remainder = chonky_boytes.remainder();
        for item in chonky_boytes {
            let i = i32::from_le_bytes(<[u8; 4]>::try_from(item).expect("unreachable"));
            self.push(i);
        }
        let mut remaining_chonk = [0u8; 4];
        remaining_chonk[0..remainder.len()].copy_from_slice(remainder);
        self.push(i32::from_le_bytes(remaining_chonk));
        self.push(valid_bytes_len as i32);
    }

    pub fn get_str(&mut self) -> &str {
        let stack = &mut self.data;
        let string_bytes_len = stack.pop().unwrap() as usize;
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
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
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
    vm.push_op(Ops::FFI(FFI::ConsoleLog1));

    vm.push_op(Ops::Pop);

    vm.push(5);
    vm.push_op(Ops::Add);

    vm.push(10);
    vm.push(20);
    vm.push_op(Ops::Mul);

    let ser: heapless::Vec<u8, 32> = postcard::to_vec(&vm)?;
    let mut de: VM = postcard::from_bytes(&ser)?;

    de.dump_state();
    while let Ok(_) = de.step() {
        de.dump_state();
    }

    Ok(())
}
