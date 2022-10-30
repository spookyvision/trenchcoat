use std::{collections::HashMap, str::from_utf8};

use dioxus::prelude::*;
use fermi::{use_atom_state, use_read, Atom, AtomState};
use gloo::timers::future::TimeoutFuture;
use runtime::WebRuntime;
use trenchcoat::{
    forth::{
        compiler::{compile, Compiler, Flavor, MockRuntime},
        vm::VM,
    },
    pixelblaze::{
        executor::Executor,
        ffi::{PixelBlazeFFI, FFI_FUNCS},
        runtime::ConsoleRuntime,
        traits::PixelBlazeRuntime,
    },
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

fn trenchit(pixel_count: usize) -> anyhow::Result<WebExecutor> {
    let js = include_str!("../../res/rainbow melt.js");

    let mut ser = compile(js, Flavor::Pixelblaze)?;
    let mut vm: VM<PixelBlazeFFI, WebRuntime> = postcard::from_bytes_cobs(&mut ser)?;

    let pixel_count = 20;
    vm.runtime_mut().init(pixel_count);
    let mut executor = Executor::new(vm, pixel_count);

    executor.start();
    return Ok(executor);
    // Err(())
}

#[allow(non_snake_case)]
#[inline_props]
fn Pb(cx: Scope) -> Element {
    let executor_state = use_atom_state(&cx, EXECUTOR);
    let mut content = rsx!("something's missing");

    if let Some(executor) = executor_state.get() {
        if let Some(runtime) = executor.runtime() {
            if let Some(leds) = runtime.leds() {
                let inner = leds.iter().cloned().enumerate().map(|(led_id, led)| {
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
        }
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
