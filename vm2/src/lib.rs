use std::collections::HashMap;

use shrinkwraprs::Shrinkwrap;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

struct FFIDef {}

enum CallTarget {
    Native(Stack),
    FFI(String),
}

enum Op {
    Call(String),
    Add,
}

enum Cell {
    Op(Op),
    Val(i32),
}

#[derive(Shrinkwrap)]
struct Stack(Vec<Op>);

enum StackError {
    InvalidState,
    Empty,
}

enum FFIA {
    ConsoleLog,
}

enum FFIB {
    Leds,
}

impl Stack {
    // fn pop_string(&mut self) -> Result<String, StackError> {Ok(())}
    // fn pop_val(&mut self) -> Result<i32, StackError> {Ok(())}
}

pub enum Param {
    Cell,
    StackString,
}

// TODO evaluate bevy_reflect
trait FFIdef {
    fn call_info(&self) -> &[Param];
}

struct VM {
    funcs: HashMap<String, Stack>,
    ffis: HashMap<String, Box<dyn FFIdef>>,
    stack: Stack,
}

impl VM {
    pub fn register_func<F, T>(f: F)
    where
        F: FnMut(&T),
    {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
