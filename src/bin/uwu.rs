use std::{collections::HashMap, path::Path};

use pixelblaze_rs::{
    forth::vis0r::Vis0r,
    pixelblaze::{executor::Executor, funcs::FFI_FUNCS, runtime::ConsoleRuntime},
};
use swc_common::{
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::Visit;

// what's this?
fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    // let fm = cm.new_source_file(FileName::Custom("fake file.js".into()), js.into());

    let fm = cm
        .load_file(Path::new("res/rainbow melt.js"))
        .expect("failed to load");
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
        let mut v = Vis0r::new(
            FFI_FUNCS
                .into_iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect::<HashMap<_, _>>(),
        );
        dbg!(&module);
        v.visit_module(&module);
        return Ok(());

        let vm = v.into_vm(ConsoleRuntime::default());
        let pixel_count = 4;

        let mut executor = Executor::new(vm, pixel_count);
        executor.start();
        executor.do_frame();
        executor.do_frame();
        println!("\n*** DÖNE ***\n");
    }

    Ok(())
}
