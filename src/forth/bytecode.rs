use core::str::from_utf8;
use std::{f64::consts::PI, time::Instant};

use fixed::{traits::ToFixed, types::extra::U8, FixedI32};
use log::trace;
use serde::{Deserialize, Serialize};

use super::pixelblaze::{abs, hsv, time, wave};

pub type VarString = heapless::String<16>;
pub type Map<K, V, const N: usize> = heapless::FnvIndexMap<K, V, N>;
// TODO pixelblaze uses <16,16> but that's not the best general range
// -> add flavors
pub type CellData = FixedI32<U8>;
pub type VarStorage = Map<VarString, Option<CellData>, 32>;

pub type Stack<const N: usize> = heapless::Vec<Cell, N>;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Op {
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
    FFI(FFI),
    Call(VarString),
    DeclVar(VarString),
    SetVar(VarString),
    GetVar(VarString),
}

type BinOp = fn(CellData, CellData) -> CellData;

fn err(s: &str) {
    panic!("ERR: {s}")
}
impl Op {
    fn binary_op<T, P>(vm: &mut VM<T, P>, op: BinOp)
    where
        T: TimerMs,
    {
        // TODO error propagation ("done" should not be an error...)
        vm.run().ok();
        let y = vm.pop().unwrap_val();
        vm.run().ok();
        let x = vm.pop().unwrap_val();

        vm.push(Cell::Val(op(x, y)));
    }
    fn eval<T, P>(&self, vm: &mut VM<T, P>)
    where
        T: TimerMs,
    {
        // println!("----");
        // println!("eval {self:?}");
        match self {
            Op::ExitFn => {
                vm.exit_fn();
            }
            Op::Return => {
                vm.do_return();
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
                    console_log(str.as_ref());
                }
                FFI::Sin => {
                    vm.run().ok();
                    let top = vm.pop();
                    let res = cordic::sin(top.unwrap_val());
                    vm.push(Cell::Val(res));
                }
                FFI::Time => {
                    vm.run().ok();
                    let top = vm.pop();
                    let res = time(top.unwrap_val(), vm);
                    vm.push(Cell::Val(res));
                }
                FFI::Wave => {
                    vm.run().ok();
                    let top = vm.pop();
                    let res = wave(top.unwrap_val());
                    vm.push(Cell::Val(res));
                }
                FFI::Abs => {
                    vm.run().ok();
                    let top = vm.pop();
                    let res = abs(top.unwrap_val());
                    vm.push(Cell::Val(res));
                }
                FFI::Hsv => {
                    vm.run().ok();
                    let v = vm.pop().unwrap_val();

                    vm.run().ok();
                    let s = vm.pop().unwrap_val();

                    vm.run().ok();
                    let h = vm.pop().unwrap_val();

                    hsv(h, s, v);
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
            Op::DeclVar(name) => {
                vm.decl_var(name);
            }
        }

        // vm.dump_state();
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Cell {
    Val(CellData),
    Op(Op),
    Null,
}

impl<TF> From<TF> for Cell
where
    TF: ToFixed,
{
    fn from(val: TF) -> Self {
        Self::Val(CellData::from_num(val))
    }
}

impl From<Op> for Cell {
    fn from(op: Op) -> Self {
        Self::Op(op)
    }
}

impl Cell {
    fn val(num: impl ToFixed) -> Self {
        Self::Val(num.to_fixed())
    }
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
            Cell::Null => panic!("tried to read null"),
        }
    }
}

pub trait TimerMs {
    fn time_millis(&self) -> u32;
}

pub struct StdTimer {
    start: Instant,
}

impl Default for StdTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl StdTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

pub trait Peripherals {
    fn led_begin(&mut self) {}
    fn led_hsv(&mut self, idx: CellData, h: CellData, s: CellData, v: CellData);
    fn led_commit(&mut self) {}
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ConsolePeripherals;

impl Peripherals for ConsolePeripherals {
    fn led_begin(&mut self) {
        println!("LED begin");
    }

    fn led_commit(&mut self) {
        println!("LED commit");
    }

    fn led_hsv(&mut self, idx: CellData, h: CellData, s: CellData, v: CellData) {
        println!("LED[{idx}] HSV({h},{s},{v})");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FuncDef {
    stack: Stack<64>,
    params: heapless::Vec<VarString, 4>,
}

impl FuncDef {
    fn new<P: AsRef<str>>(stack: Stack<64>, params: &[P]) -> Self {
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
pub struct VM<TIMER, PERI> {
    stack: Stack<64>,
    return_stack: Stack<4>,
    return_addr: Option<usize>,
    globals: VarStorage,
    locals: heapless::Vec<VarStorage, 8>,
    funcs: Map<VarString, FuncDef, 8>,
    #[serde(skip)]
    timer: TIMER,
    #[serde(skip)]
    peripherals: PERI,
}

impl TimerMs for StdTimer {
    fn time_millis(&self) -> u32 {
        Instant::now().duration_since(self.start).as_millis() as u32
    }
}

impl<TIMER, PERI> VM<TIMER, PERI>
where
    TIMER: TimerMs,
{
    pub fn new(timer: TIMER, peripherals: PERI) -> Self {
        Self {
            stack: heapless::Vec::new(),
            return_stack: heapless::Vec::new(),
            return_addr: None,
            globals: Map::new(),
            locals: heapless::Vec::new(),
            funcs: Map::new(),
            timer,
            peripherals,
        }
    }
    pub fn time_millis(&self) -> u32 {
        self.timer.time_millis()
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

    pub fn add_func<P: AsRef<str>>(&mut self, name: impl AsRef<str>, params: &[P], stack: &[Cell]) {
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

        let context = self.locals.last_mut().unwrap_or(&mut self.globals);
        context
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
        // let name = name.as_ref().into();

        // let context = self.locals.last_mut().unwrap_or(&mut self.globals);
        // context
        //     .insert(name, Some(val))
        //     .expect("variable space exhausted");
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

    pub fn push(&mut self, i: Cell) {
        // println!("push {i:?}");
        if let Err(e) = self.stack.push(i) {
            err("stack too full");
        }
    }

    pub fn pop(&mut self) -> Cell {
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

    pub fn exit_fn(&mut self) {
        let ret = self.return_addr.take().expect("there's no return");
        self.stack.truncate(ret);
    }

    pub fn do_return(&mut self) {
        let top = self.pop();
        self.return_stack.push(top).expect("return stack too full");
    }

    pub fn run(&mut self) -> Result<(), VMErr> {
        // TODO meh, would rather not clone
        while let Some(Cell::Op(op)) = self.stack.last().cloned() {
            self.stack.pop();
            trace!("running {op:?}");
            self.dump_state();
            op.eval(self);

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

    pub fn stack(&self) -> &[Cell] {
        self.stack.as_ref()
    }
}
#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum FFI {
    ConsoleLog1,
    Sin,
    Time,
    Wave,
    Abs,
    Hsv,
}

fn console_log(s: &str) {
    println!("[VM::LOG] {s}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forth::util::{assert_similar, vm};

    #[test]
    fn test_str() -> anyhow::Result<()> {
        let mut vm = VM::new(StdTimer::new(), ConsolePeripherals);
        let s = "⭐hello, vm!⭐";
        vm.push_str(s);
        assert_eq!(vm.get_str().as_ref(), s);

        Ok(())
    }

    #[test]
    fn test_ffi() -> anyhow::Result<()> {
        let mut vm = vm();
        vm.push(Cell::from(-5i32));
        vm.push(Op::FFI(FFI::Abs).into());
        vm.run();
        assert_eq!(&[Cell::from(5i32)], &vm.stack);
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
        let mut de: VM<StdTimer, ConsolePeripherals> = postcard::from_bytes(&ser)?;

        de.run();
        de.do_return();
        assert_eq!(&[Cell::val(90)], &de.return_stack);
        assert_eq!(&[], &de.stack);
        Ok(())
    }

    #[test]
    fn test_sin() -> anyhow::Result<()> {
        let mut vm = vm();

        let param = 0.1f64;
        vm.push(Cell::val(param));
        vm.push(Cell::Op(Op::FFI(FFI::Sin)));

        vm.run();

        let precise: f64 = param.sin();
        let approximate = vm.stack.pop().unwrap().unwrap_val();

        assert_similar(precise, approximate, 1);
        Ok(())
    }
}
