use std::{collections::HashMap, str::from_utf8};

use dioxus::prelude::*;
use fermi::{use_atom_state, use_read, Atom, AtomState};
use gloo::timers::future::TimeoutFuture;
use runtime::WebRuntime;
use swc_common::{
    errors::{emitter::Destination, ColorConfig, EmitterWriter, Handler},
    sync::Lrc,
    FileName, SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::Visit;
use trenchcoat::{
    forth::compiler::Compiler,
    pixelblaze::{
        executor::Executor,
        ffi::{PixelBlazeFFI, FFI_FUNCS},
        runtime::ConsoleRuntime,
        traits::PixelBlazeRuntime,
    },
    prelude::*,
};

use crate::{render::LedWidget, runtime::Led};

mod render;
mod runtime;

type WebExecutor = Executor<PixelBlazeFFI, WebRuntime>;

pub static EXECUTOR: Atom<Option<WebExecutor>> = |_| None;

#[derive(Clone, Copy)]
struct WebConsole;

impl std::io::Write for WebConsole {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(s) = from_utf8(buf) {
            log::warn!("{s}");
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn main() {
    let base_url: UseState<String>;
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    console_error_panic_hook::set_once();
    dioxus::web::launch(app);
}

fn trenchit(pixel_count: usize) -> Result<WebExecutor, ()> {
    let source_map: Lrc<SourceMap> = Default::default();
    let emitter = EmitterWriter::new(Box::new(WebConsole), Some(source_map.clone()), false, false);
    let handler = Handler::with_emitter(true, false, Box::new(emitter));

    let js = include_str!("../../res/rainbow melt.js");
    let fm = source_map.new_source_file(FileName::Custom("test.js".into()), js.into());

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
        e.into_diagnostic(&handler).emit();
    }) {
        let mut v = Compiler::new(
            FFI_FUNCS
                .into_iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect::<HashMap<_, _>>(),
        );
        v.visit_module(&module);

        let vm = v.into_vm(WebRuntime::new(pixel_count));

        let mut executor = Executor::new(vm, pixel_count);
        executor.start();
        return Ok(executor);
    }
    Err(())
}

#[allow(non_snake_case)]
#[inline_props]
fn Pb(cx: Scope) -> Element {
    let executor_state = use_atom_state(&cx, EXECUTOR);
    let mut content = rsx!("no executor?");

    if let Some(executor) = executor_state.get() {
        let inner = executor
            .runtime()
            .leds()
            .iter()
            .cloned()
            .enumerate()
            .map(|(led_id, led)| {
                rsx! {
                    div {
                        class: "square-container",
                        key: "led-{led_id}",
                        LedWidget { led: led }
                    }
                }
            });
        content = rsx!(div { inner });
    }

    cx.render(content)
}

fn app(cx: Scope) -> Element {
    let executor_state = use_atom_state(&cx, EXECUTOR).to_owned();
    let _irish_setter: &UseFuture<()> = use_future(&cx, (), |_| async move {
        loop {
            executor_state.with_mut(|executor| {
                if let Some(executor) = executor {
                    executor.do_frame();
                } else {
                    let mut i_x = trenchit(40).unwrap();
                    i_x.start();
                    i_x.do_frame();
                    *executor = Some(i_x);
                }
            });
            TimeoutFuture::new(100).await;
        }
    });

    cx.render(rsx! (
        div {
            style: "text-align: center;",
            h1 { "Yo dawg…" }
            Pb { }
        }
    ))
}
