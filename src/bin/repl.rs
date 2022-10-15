use std::io::{self, BufRead};

use pixelblaze_rs::forth;

fn to_quit(cmd: &str) -> bool {
    match cmd {
        "quit" | "q" | "exit" => true,
        _ => false,
    }
}

fn run_forth() {
    let mut env = forth::env::ForthEnv::empty();
    let intr = forth::inter::Interpreter::new();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let mut input = line.unwrap();
        input = input.trim().to_string();

        if to_quit(&input) {
            println!("Bye!");
            return;
        } else {
            intr.eval(&mut env, &input);
        }
    }
}

fn main() {
    run_forth();
}
