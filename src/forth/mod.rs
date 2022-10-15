pub mod bytecode;
pub mod env;
pub mod inter;
mod ops;

fn valid_forth_name(name: &str) -> bool {
    name.parse::<i32>().is_err()
}
