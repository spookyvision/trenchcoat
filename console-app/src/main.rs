use std::{collections::HashMap, path::Path};

use swc_common::{
    errors::{ColorConfig, Handler},
    sync::Lrc,
    SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::Visit;
use trenchcoat::{
    forth::compiler::Compiler,
    pixelblaze::{executor::Executor, ffi::FFI_FUNCS, runtime::ConsoleRuntime},
    prelude::*,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let file = "../res/rainbow melt.js";
    // let file = "../res/test_ffi.js";

    let fm = cm.load_file(Path::new(file)).expect("failed to load");
    let lexer = Lexer::new(
        // We want to parse ecmascript
        Syntax::Es(Default::default()),
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    if let Ok(module) = parser.parse_module().map_err(|e| {
        // Unrecoverable fatal error occurred
        e.into_diagnostic(&handler).emit()
    }) {
        let mut v = Compiler::new(
            FFI_FUNCS
                .into_iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect::<HashMap<_, _>>(),
        );
        dbg!(&module);
        v.visit_module(&module);

        let vm = v.into_vm(ConsoleRuntime::default());
        let pixel_count = 4;

        let mut executor = Executor::new(vm, pixel_count);
        executor.start();

        for _frame in 0..5 {
            executor.do_frame();
        }

        executor.exit();
    }

    Ok(())
}
