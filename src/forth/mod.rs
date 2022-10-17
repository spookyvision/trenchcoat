pub mod bytecode;
pub mod env;
pub mod inter;
mod ops;
mod pixelblaze;

#[cfg(test)]
pub mod util;

fn valid_forth_name(name: &str) -> bool {
    name.parse::<i32>().is_err()
}
