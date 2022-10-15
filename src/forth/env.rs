use std::{collections::HashMap, result};

pub(crate) type ForthResult<T> = result::Result<T, String>;
pub(crate) type Ops = dyn Fn(&mut ForthEnv) -> ForthResult<()>;
pub(crate) type ForthFunc = (String, Vec<String>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ForthVar {
    Var(i32),
    Array(Vec<i32>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum VarRef {
    Var(String),
    Array(String, i32),
}

pub struct ForthEnv {
    stack: Vec<i32>,
    funcs: HashMap<String, ForthFunc>,
    vars: HashMap<String, ForthVar>,
    var_refs: Vec<VarRef>,
    constants: HashMap<String, i32>,
    specials: HashMap<String, i32>,
}

impl ForthEnv {
    pub fn empty() -> ForthEnv {
        ForthEnv {
            stack: vec![],
            funcs: HashMap::new(),
            vars: HashMap::new(),
            var_refs: vec![],
            constants: HashMap::new(),
            specials: HashMap::new(),
        }
    }

    pub fn top_variable_ref(&mut self) -> Option<VarRef> {
        if self.var_refs.is_empty() {
            None
        } else {
            let var = self.var_refs[self.var_refs.len() - 1].clone();
            Some(var)
        }
    }

    pub fn pop_variable_ref(&mut self) -> Option<VarRef> {
        self.var_refs.pop()
    }

    pub fn push_variable_ref(&mut self, var: VarRef) {
        self.var_refs.push(var);
    }

    pub fn get_variable(&self, name: &str) -> Option<ForthVar> {
        self.vars.get(name).cloned()
    }

    pub fn add_variable(&mut self, name: &str, value: ForthVar) -> Option<ForthVar> {
        self.vars.insert(name.to_string(), value)
    }

    pub fn get_constant(&self, name: &str) -> Option<i32> {
        self.constants.get(name).cloned()
    }

    pub fn add_constant(&mut self, name: &str, value: i32) -> Option<i32> {
        self.constants.insert(name.to_string(), value)
    }

    pub fn get_function(&self, name: &str) -> Option<ForthFunc> {
        match self.funcs.get(name) {
            Some(func) => Some(func.clone()),
            None => None,
        }
    }

    pub fn add_function(&mut self, name: &str, func: ForthFunc) -> Option<ForthFunc> {
        self.funcs.insert(name.to_string(), func)
    }

    pub fn pop(&mut self, msg: String) -> ForthResult<i32> {
        match self.stack.pop() {
            Some(n) => Ok(n),
            None => Err(msg),
        }
    }

    pub fn top(&mut self, msg: String) -> ForthResult<i32> {
        match self.stack.len() {
            0 => Err(msg),
            n => Ok(self.stack[n - 1]),
        }
    }

    pub fn push(&mut self, val: i32) {
        self.stack.push(val);
    }

    pub fn print_stack(&self) {
        println!("{:?}", self.stack);
    }

    pub fn print_func(&self) {
        println!("{:?}", self.funcs);
    }

    pub fn print_vars(&self) {
        println!("{:?}", self.vars);
    }

    pub fn get_special(&self, name: &str) -> Option<i32> {
        if self.specials.contains_key(name) {
            let val = *self.specials.get(name).unwrap();
            Some(val)
        } else {
            None
        }
    }

    pub fn set_special(&mut self, name: &str, value: i32) -> Option<i32> {
        self.specials.insert(name.to_string(), value)
    }

    pub fn clear_special(&mut self, name: &str) -> Option<i32> {
        self.specials.remove(name)
    }

    pub fn allot_array(&mut self, name: &str, length: i32) {
        let arr = ForthVar::Array(vec![0; length as usize]);
        self.vars.insert(name.to_string(), arr);
    }

    pub fn array_set(&mut self, name: &str, pos: i32, value: i32) -> ForthResult<()> {
        if self.vars.contains_key(name) {
            let var = self.vars.get(name).unwrap().clone();
            match var {
                ForthVar::Var(_) => Err(format!("Variable {} is not an array!", name)),
                ForthVar::Array(vec) => {
                    let mut new_vec = vec.clone();
                    if new_vec.len() > (pos as usize) && pos >= 0 {
                        new_vec[pos as usize] = value;
                        self.vars.insert(name.to_string(), ForthVar::Array(new_vec));
                        Ok(())
                    } else {
                        Err(format!(
                            "Cannot set at position: {} when array length is: {}",
                            pos,
                            new_vec.len()
                        ))
                    }
                }
            }
        } else {
            Err(format!("No such array variable: {}", name))
        }
    }
}
