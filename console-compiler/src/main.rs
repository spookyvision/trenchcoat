use std::{collections::HashMap, fs::File, io::Write};

use anyhow::{anyhow, Context};
use clap::Parser;
use swc_common::{
    errors::{ColorConfig, Handler},
    sync::Lrc,
    SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, Parser as EcmaParser, StringInput, Syntax};
use swc_ecma_visit::{swc_ecma_ast::Module, Visit};
use trenchcoat::{
    forth::{compiler::Compiler, vm::FFIOps},
    pixelblaze::{self, runtime::ConsoleRuntime},
    prelude::*,
};

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq)]
enum Flavor {
    VanillaJS,
    Pixelblaze,
}

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

    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let file = &args.in_file.into_boxed_path();

    let fm = cm
        .load_file(file)
        .with_context(|| format!("Failed to load {file:?}"))?;
    let lexer = Lexer::new(
        // We want to parse ecmascript
        Syntax::Es(Default::default()),
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = EcmaParser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    if let Ok(module) = parser.parse_module().map_err(|e| {
        e.clone().into_diagnostic(&handler).emit();
    }) {
        let ser = match args.flavor {
            Flavor::VanillaJS => todo!(),
            Flavor::Pixelblaze => compile(
                module,
                pixelblaze::ffi::FFI_FUNCS,
                ConsoleRuntime::default(),
            ),
        }
        .map_err(|e| anyhow!("Compilation failed: {e:?}"))?;

        File::create(args.out_file)?.write_all(&ser)?;
        return Ok(());
    }

    anyhow::bail!("Compilation failed")
}

fn compile<FFI, RT>(
    module: Module,
    ffi_defs: phf::Map<&str, FFI>,
    runtime: RT,
) -> Result<Vec<u8>, postcard::Error>
where
    FFI: FFIOps<RT> + Copy + Eq + serde::Serialize,
    RT: Clone + PartialEq,
{
    let mut v = Compiler::new(
        ffi_defs
            .into_iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect::<HashMap<_, _>>(),
    );
    v.visit_module(&module);

    let vm = v.into_vm(runtime);
    println!("vm size is {}", std::mem::size_of_val(&vm));
    postcard::to_allocvec_cobs(&vm)
}
