use std::{fs::File, io::Write};

use clap::Parser;
use trenchcoat::forth::compiler::{compile, Flavor};

/// Trenchcoat bytecode compiler
#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source code flavor
    #[arg(short, long)]
    flavor: Flavor,

    /// Input file
    #[arg(short, long)]
    in_file: std::path::PathBuf,

    /// Output file (.tcb)
    #[arg(short, long)]
    out_file: std::path::PathBuf,
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let args = Args::parse();
    let file = args.in_file.into_boxed_path();
    let ser = compile(&file, args.flavor)?;
    File::create(args.out_file)?.write_all(&ser)?;
    Ok(())
}
