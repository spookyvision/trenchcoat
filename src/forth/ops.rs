use crate::forth::env::{ForthEnv, ForthResult};

// Binary operations
type BinOp = fn(i32, i32) -> i32;

fn binary_op(name: &str, op: BinOp, env: &mut ForthEnv) -> ForthResult<()> {
    let x = env.pop(format!(
        "Empty stack: for first argument for {}",
        name.to_string()
    ))?;
    let y = env.pop(format!(
        "Empty stack: for second argument for {}",
        name.to_string()
    ))?;
    env.push(op(x, y));
    Ok(())
}

pub fn add(env: &mut ForthEnv) -> ForthResult<()> {
    binary_op("+", |x, y| x + y, env)
}

pub fn subtract(env: &mut ForthEnv) -> ForthResult<()> {
    binary_op("-", |x, y| y - x, env)
}

pub fn mul(env: &mut ForthEnv) -> ForthResult<()> {
    binary_op("*", |x, y| x * y, env)
}

pub fn div(env: &mut ForthEnv) -> ForthResult<()> {
    binary_op("/", |x, y| y / x, env)
}

pub fn modulus(env: &mut ForthEnv) -> ForthResult<()> {
    binary_op("mod", |x, y| y % x, env)
}

pub fn and(env: &mut ForthEnv) -> ForthResult<()> {
    binary_op("and", |x, y| y & x, env)
}

pub fn or(env: &mut ForthEnv) -> ForthResult<()> {
    binary_op("or", |x, y| y | x, env)
}

// Core operations
pub fn dup(env: &mut ForthEnv) -> ForthResult<()> {
    let x = env.pop("Empty stack for dup".to_string())?;
    env.push(x);
    env.push(x);
    Ok(())
}

pub fn pop(env: &mut ForthEnv) -> ForthResult<()> {
    let x = env.pop("Empty stack for .".to_string())?;
    println!("{}", x);
    Ok(())
}

pub fn swap(env: &mut ForthEnv) -> ForthResult<()> {
    let x = env.pop("Empty stack for first element in swap".to_string())?;
    let y = env.pop("Empty stack for second element in swap".to_string())?;
    env.push(x);
    env.push(y);
    Ok(())
}

pub fn over(env: &mut ForthEnv) -> ForthResult<()> {
    let x = env.pop("Empty stack for first element in over".to_string())?;
    let y = env.pop("Empty stack for second element in over".to_string())?;
    env.push(y);
    env.push(x);
    env.push(y);
    Ok(())
}

pub fn rot(env: &mut ForthEnv) -> ForthResult<()> {
    let x = env.pop("Empty stack for first element in rot".to_string())?;
    let y = env.pop("Empty stack for second element in rot".to_string())?;
    let z = env.pop("Empty stack for third element in rot".to_string())?;
    env.push(y);
    env.push(x);
    env.push(z);
    Ok(())
}

pub fn drop(env: &mut ForthEnv) -> ForthResult<()> {
    env.pop("Empty stack for drop".to_string())?;
    Ok(())
}

pub fn emit(env: &mut ForthEnv) -> ForthResult<()> {
    let x = env.pop("Empty stack for emit".to_string())?;
    print!("{}", (x as u8) as char);
    Ok(())
}

pub fn cr(_: &mut ForthEnv) -> ForthResult<()> {
    println!();
    Ok(())
}

pub fn print_stack(env: &mut ForthEnv) -> ForthResult<()> {
    print!("Stack: ");
    env.print_stack();
    Ok(())
}

pub fn print_func(env: &mut ForthEnv) -> ForthResult<()> {
    print!("Dictionary: ");
    env.print_func();
    Ok(())
}

pub fn print_vars(env: &mut ForthEnv) -> ForthResult<()> {
    print!("Variables: ");
    env.print_vars();
    Ok(())
}

// Boolean operations
type BinBoolOp = fn(i32, i32) -> bool;

fn binary_bool_op(name: &str, op: BinBoolOp, env: &mut ForthEnv) -> ForthResult<()> {
    let x = env.pop(format!(
        "Empty stack: for first argument for {}",
        name.to_string()
    ))?;
    let y = env.pop(format!(
        "Empty stack: for second argument for {}",
        name.to_string()
    ))?;
    if op(x, y) {
        env.push(-1);
    } else {
        env.push(0);
    }
    Ok(())
}

pub fn eq(env: &mut ForthEnv) -> ForthResult<()> {
    binary_bool_op("=", |x, y| x == y, env)
}

pub fn not_eq(env: &mut ForthEnv) -> ForthResult<()> {
    binary_bool_op("=", |x, y| x != y, env)
}

pub fn lt(env: &mut ForthEnv) -> ForthResult<()> {
    binary_bool_op("<", |x, y| y < x, env)
}

pub fn gt(env: &mut ForthEnv) -> ForthResult<()> {
    binary_bool_op(">", |x, y| y > x, env)
}

pub fn lt_eq(env: &mut ForthEnv) -> ForthResult<()> {
    binary_bool_op("<=", |x, y| y <= x, env)
}

pub fn gt_eq(env: &mut ForthEnv) -> ForthResult<()> {
    binary_bool_op(">=", |x, y| y >= x, env)
}

pub fn invert(env: &mut ForthEnv) -> ForthResult<()> {
    let x = env.pop("Empty stack for invert".to_string())?;
    env.push(if x == 0 { -1 } else { 0 });
    Ok(())
}
