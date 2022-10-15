use std::{collections::HashMap, slice::Iter};

use crate::forth::{
    env::{ForthEnv, ForthFunc, ForthResult, ForthVar, Ops, VarRef},
    ops, valid_forth_name,
};

pub struct Interpreter<'a> {
    pub(crate) builtins: HashMap<String, &'a Ops>,
}

fn rs_add(a: i32, b: i32) -> i32 {
    a + b
}

impl<'a> Interpreter<'a> {
    pub fn eval(&self, env: &mut ForthEnv, expr: &str) {
        let tokens: Vec<_> = expr.split(' ').map(|s| s.trim().to_string()).collect();
        self.eval_toks(env, &mut tokens.iter());
    }

    pub fn new() -> Self {
        let mut intr = Interpreter {
            builtins: HashMap::new(),
        };

        intr.init();
        intr
    }

    fn init(&mut self) {
        // Binary ops
        self.builtins.insert("+".to_owned(), &ops::add);
        self.builtins.insert("-".to_owned(), &ops::subtract);
        self.builtins.insert("*".to_owned(), &ops::mul);
        self.builtins.insert("/".to_owned(), &ops::div);
        self.builtins.insert("mod".to_owned(), &ops::modulus);
        self.builtins.insert("and".to_owned(), &ops::and);
        self.builtins.insert("or".to_owned(), &ops::or);

        // Core ops
        self.builtins.insert("p".to_owned(), &ops::print_stack);
        self.builtins.insert("d".to_owned(), &ops::print_func);
        self.builtins.insert("v".to_owned(), &ops::print_vars);
        self.builtins.insert("dup".to_owned(), &ops::dup);
        self.builtins.insert(".".to_owned(), &ops::pop);
        self.builtins.insert("drop".to_owned(), &ops::drop);
        self.builtins.insert("swap".to_owned(), &ops::swap);
        self.builtins.insert("over".to_owned(), &ops::over);
        self.builtins.insert("rot".to_owned(), &ops::rot);
        self.builtins.insert("emit".to_owned(), &ops::emit);
        self.builtins.insert("cr".to_owned(), &ops::cr);

        // Boolean ops
        self.builtins.insert("=".to_owned(), &ops::eq);
        self.builtins.insert("!=".to_owned(), &ops::not_eq);
        self.builtins.insert("<".to_owned(), &ops::lt);
        self.builtins.insert(">".to_owned(), &ops::gt);
        self.builtins.insert("<=".to_owned(), &ops::lt_eq);
        self.builtins.insert(">=".to_owned(), &ops::gt_eq);
        self.builtins.insert("invert".to_owned(), &ops::invert);
    }

    fn eval_builtin(&self, op: &str, env: &mut ForthEnv) -> Option<ForthResult<()>> {
        if self.builtins.contains_key(op) {
            let opr = self.builtins.get(op).unwrap();
            Some(opr(env))
        } else {
            None
        }
    }

    fn eval_function(&self, name: &str, env: &mut ForthEnv) -> Option<ForthResult<()>> {
        match env.get_function(name) {
            Some(ref func) => {
                self.eval_toks(env, &mut func.1.iter());
                Some(Ok(()))
            }
            None => None,
        }
    }

    fn eval_ffi(&self, name: &str, env: &mut ForthEnv) -> Option<ForthResult<()>> {
        if name == "rs+" {
            let a = env.pop("ffi".to_string());
            let b = env.pop("ffi".to_string());
            if let (Ok(a), Ok(b)) = (a, b) {
                env.push(rs_add(a, b));
                Some(Ok(()))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn eval_variable(&self, name: &str, env: &mut ForthEnv) -> Option<ForthResult<()>> {
        match env.get_variable(name) {
            Some(var) => {
                match var {
                    ForthVar::Var(_) => env.push_variable_ref(VarRef::Var(name.to_string())),
                    ForthVar::Array(_) => env.push_variable_ref(VarRef::Array(name.to_string(), 0)),
                };
                Some(Ok(()))
            }
            None => None,
        }
    }

    fn eval_constant(&self, name: &str, env: &mut ForthEnv) -> Option<ForthResult<()>> {
        if let Some(x) = env.get_constant(&name) {
            env.push(x);
            Some(Ok(()))
        } else {
            None
        }
    }

    fn eval_special_variables(&self, name: &str, env: &mut ForthEnv) -> bool {
        match env.get_special(name) {
            Some(val) => {
                env.push(val);
                true
            }
            None => false,
        }
    }

    pub fn eval_toks(&self, env: &mut ForthEnv, toks: &mut Iter<String>) {
        while let Some(s) = toks.next() {
            // for s in &toks {
            if s.trim().is_empty() {
                // skip empty tokens
                continue;
            }

            // Handle special forms
            match self.eval_special_forms(s, env, toks) {
                None => (),
                Some(Ok(())) => continue,
                Some(Err(e)) => {
                    println!("Error: {}", e);
                    break;
                }
            }

            // Handle special variables
            if self.eval_special_variables(s, env) {
                continue;
            }

            match self.eval_builtin(s, env) {
                None => (),
                Some(Ok(())) => continue,
                Some(Err(e)) => {
                    println!("Error: {}", e);
                    break;
                }
            }

            match self.eval_function(s, env) {
                None => (),
                Some(Ok(())) => continue,
                Some(Err(e)) => {
                    println!("Error: {}", e);
                    break;
                }
            }

            match self.eval_ffi(s, env) {
                None => (),
                Some(Ok(())) => continue,
                Some(Err(e)) => {
                    println!("Error: {}", e);
                    break;
                }
            }

            // Handle as a number
            match s.parse::<i32>() {
                Ok(num) => env.push(num),
                Err(_) => {
                    // Check if this is a valid variable or not
                    match self.eval_constant(s, env) {
                        None => match self.eval_variable(s, env) {
                            None => {
                                println!("Invalid token: {}", s);
                                break;
                            }
                            Some(Ok(())) => continue,
                            Some(Err(e)) => {
                                println!("Error: {}", e);
                                break;
                            }
                        },
                        Some(Ok(())) => continue,
                        Some(Err(e)) => {
                            println!("Error: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        print!("=> ");
        env.print_stack();
    }

    fn parse_function(&self, toks: &mut Iter<String>) -> ForthResult<ForthFunc> {
        // Get the name
        if let Some(name) = toks.next() {
            if !valid_forth_name(name) {
                return Err(format!("Invalid name for function: {}", name));
            }
            let mut definition: Vec<String> = vec![];

            for s in toks {
                if s == ";" {
                    // end of function definition
                    return Ok((name.to_string(), definition));
                } else {
                    definition.push(s.to_string());
                }
            }
        }

        Err("Invalid function".to_string())
    }

    fn parse_string(&self, toks: &mut Iter<String>) -> ForthResult<String> {
        let mut msg = String::new();

        for m in toks {
            if m == "\"" {
                return Ok(msg);
            }
            if m.ends_with('"') {
                let mut temp = m.clone();
                temp.pop();
                msg.push_str(&temp.clone());
                return Ok(msg);
            }
            msg.push_str(&format!("{} ", m));
        }

        Err("Nonterminated string".to_string())
    }

    fn eval_conditional(&self, env: &mut ForthEnv, toks: &mut Iter<String>) -> ForthResult<()> {
        let res = env.pop("Empty stack for condition in if".to_string())?;

        // Check the top of the stack and if it is not zero then evaluate till then
        let mut before_else = vec![];
        let mut after_else = vec![];
        let mut found_else = false;

        for t in toks {
            if t == "then" {
                break;
            }

            if t == "else" {
                found_else = true;
                continue;
            }

            if found_else {
                after_else.push(t.clone());
            } else {
                before_else.push(t.clone());
            }
        }

        if before_else.is_empty() {
            return Err("Empty statement for then clause".to_string());
        }
        if found_else && after_else.is_empty() {
            return Err("Empty statement for then clause after else".to_string());
        }

        if res == 0 {
            self.eval_toks(env, &mut after_else.iter());
        } else {
            self.eval_toks(env, &mut before_else.iter());
        }

        Ok(())
    }

    fn eval_special_forms(
        &self,
        start: &str,
        env: &mut ForthEnv,
        toks: &mut Iter<String>,
    ) -> Option<ForthResult<()>> {
        // Handle function definitions
        if start == ":" {
            match self.parse_function(toks) {
                Ok(func) => {
                    println!("Defined: {:?}", func);
                    let fname = func.0.clone();
                    env.add_function(&fname, func);
                    return Some(Ok(()));
                }
                Err(e) => return Some(Err(e)),
            }
        }

        // Handle string output
        if start == ".\"" {
            match self.parse_string(toks) {
                Ok(msg) => {
                    print!("{}", msg);
                    return Some(Ok(()));
                }
                Err(e) => return Some(Err(e)),
            }
        }

        if start == "if" {
            match self.eval_conditional(env, toks) {
                Ok(()) => return Some(Ok(())),
                Err(e) => return Some(Err(e)),
            }
        }

        if start == "@" {
            // Variable get
            match self.eval_variable_get(env) {
                Ok(()) => return Some(Ok(())),
                Err(e) => return Some(Err(e)),
            }
        }

        if start == "!" {
            // Variable set
            match self.eval_variable_set(env) {
                Ok(()) => return Some(Ok(())),
                Err(e) => return Some(Err(e)),
            }
        }

        if start == "variable" {
            // Variable introduction
            match self.eval_intro_variable(env, toks) {
                Ok(()) => return Some(Ok(())),
                Err(e) => return Some(Err(e)),
            }
        }

        if start == "constant" {
            // Variable introduction
            match self.eval_intro_constant(env, toks) {
                Ok(()) => return Some(Ok(())),
                Err(e) => return Some(Err(e)),
            }
        }

        if start == "cells" {
            match self.eval_cells(env) {
                Ok(()) => return Some(Ok(())),
                Err(e) => return Some(Err(e)),
            }
        }

        if start == "allot" {
            match self.eval_allot(env) {
                Ok(()) => return Some(Ok(())),
                Err(e) => return Some(Err(e)),
            }
        }

        if start == "do" {
            // do loop
            match self.eval_do_loop(env, toks) {
                Ok(()) => return Some(Ok(())),
                Err(e) => return Some(Err(e)),
            }
        }

        if start == "+" {
            match self.eval_set_array_slot(env) {
                Some(Ok(())) => return Some(Ok(())),
                Some(Err(e)) => return Some(Err(e)),
                None => (),
            }
        }

        None
    }

    fn eval_set_array_slot(&self, env: &mut ForthEnv) -> Option<ForthResult<()>> {
        if let Some(variable) = env.top_variable_ref() {
            match variable {
                VarRef::Var(name) => {
                    Some(Err(format!("Cannot set slot on normal variable: {}", name)))
                }
                VarRef::Array(name, _) => {
                    match env.pop("Stack empty to set slot value for array".to_string()) {
                        Ok(pos) => {
                            env.pop_variable_ref();
                            env.push_variable_ref(VarRef::Array(name, pos));
                            Some(Ok(()))
                        }
                        Err(e) => Some(Err(e)),
                    }
                }
            }
        } else {
            None
        }
    }

    fn eval_allot(&self, env: &mut ForthEnv) -> ForthResult<()> {
        match env.pop_variable_ref() {
            Some(VarRef::Var(name)) => {
                let length = env.pop("Stack empty to allot array".to_string())?;
                env.allot_array(&name, length);
                Ok(())
            }
            Some(VarRef::Array(name, _)) => Err(format!("{} is already an array", name)),
            None => Err("No variable found to allocate as an array!".to_string()),
        }
    }

    fn eval_cells(&self, env: &mut ForthEnv) -> ForthResult<()> {
        // We don't have actually do anything but to ensure that the stack is
        // not empty to preserve Forth semantics
        env.top("Empty stack to evaluate cells".to_string())?;
        Ok(())
    }

    fn eval_intro_variable(&self, env: &mut ForthEnv, toks: &mut Iter<String>) -> ForthResult<()> {
        if let Some(var_name) = toks.next() {
            if valid_forth_name(var_name) {
                env.add_variable(var_name, ForthVar::Var(0));
                env.push_variable_ref(VarRef::Var(var_name.clone()));
                Ok(())
            } else {
                Err(format!("Invalid variable name: {}", var_name))
            }
        } else {
            Err("Variable name not found".to_string())
        }
    }

    // do [loop body] loop
    fn eval_do_loop(&self, env: &mut ForthEnv, toks: &mut Iter<String>) -> ForthResult<()> {
        let mut loop_body = vec![];

        for t in toks {
            if t == "loop" {
                break;
            }
            loop_body.push(t.clone());
        }

        if loop_body.is_empty() {
            return Err("Empty loop body".to_string());
        }

        let start = env.pop("Empty stack for start of do loop".to_string())?;
        let end = env.pop("Empty stack for end of do loop".to_string())?;

        for i in start..end {
            env.set_special("i", i);
            self.eval_toks(env, &mut loop_body.iter());
        }

        env.clear_special("i");

        Ok(())
    }

    fn eval_intro_constant(&self, env: &mut ForthEnv, toks: &mut Iter<String>) -> ForthResult<()> {
        if let Some(const_name) = toks.next() {
            let x = env.pop(format!("Stack empty to set constant {}", const_name))?;
            env.add_constant(const_name, x);
            Ok(())
        } else {
            Err("Variable name not found".to_string())
        }
    }

    fn eval_variable_set(&self, env: &mut ForthEnv) -> ForthResult<()> {
        match env.pop_variable_ref() {
            Some(var) => match var {
                VarRef::Var(var_name) => {
                    let x = env.pop("Stack empty to set variable value".to_string())?;
                    match env.add_variable(&var_name, ForthVar::Var(x)) {
                        Some(_) => Ok(()),
                        None => Err(format!("No such variable: {}", var_name)),
                    }
                }
                VarRef::Array(var_name, pos) => {
                    let x = env.pop("Stack empty to set array value".to_string())?;

                    if env.get_variable(&var_name).is_some() {
                        match env.array_set(&var_name, pos, x) {
                            Ok(()) => Ok(()),
                            Err(e) => Err(format!(
                                "Setting array {} value failed because: {}",
                                var_name, e
                            )),
                        }
                    } else {
                        Err(format!("No such array: {}", var_name))
                    }
                }
            },
            None => Err("No variable reference found to set value".to_string()),
        }
    }

    fn eval_variable_get(&self, env: &mut ForthEnv) -> ForthResult<()> {
        match env.pop_variable_ref() {
            Some(VarRef::Var(var_name)) => match env.get_variable(&var_name) {
                Some(ForthVar::Var(num)) => {
                    env.push(num);
                    Ok(())
                }
                Some(ForthVar::Array(_)) => Err(format!(
                    "Unexpected array found when variable expected with name {}",
                    var_name
                )),
                None => Err(format!("No such variable: {}", var_name)),
            },
            Some(VarRef::Array(var_name, pos)) => match env.get_variable(&var_name).unwrap() {
                ForthVar::Var(_) => Err(format!(
                    "Unexpected variable found when array expected with name {}",
                    var_name
                )),
                ForthVar::Array(arr) => {
                    let x = arr[pos as usize];
                    env.push(x);
                    Ok(())
                }
            },
            None => Err("No variable reference found to set value".to_string()),
        }
    }
}
