use std::{collections::HashMap, str::from_utf8};

use dioxus::{core::to_owned, prelude::*};
use fermi::{use_atom_state, use_read, Atom, AtomState};
use futures::StreamExt;
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
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    console_error_panic_hook::set_once();
    dioxus::web::launch(app);
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
    let executor_state = use_atom_state(&cx, EXECUTOR);
    let js = use_state(&cx, || {
        include_str!("../../res/rainbow melt.js").to_string()
    });

    let js_ = js.clone();

    let send_to_mcu = use_coroutine(&cx, |mut rx: UnboundedReceiver<String>| async move {
        // TODO 420 surf it
        while let Some(msg) = rx.next().await {
            log::debug!("code updated: {msg}");

            if let Ok(mut ser) = compile(&msg, Flavor::Pixelblaze) {
                let url = "http://localhost:8008/";
                surf::post(url)
                    .content_type("multipart/form-data")
                    .body_bytes(&ser)
                    .await;
            }
        }
    });

    let executor_state = use_atom_state(&cx, EXECUTOR);
    let dog = use_future(
        &cx,
        (js, executor_state),
        |(js, executor_state)| async move {
            log::debug!("666 trench it {}", js);
            if let Ok(mut ser) = compile(js.as_str(), Flavor::Pixelblaze) {
                let mut next_vm: VM<PixelBlazeFFI, WebRuntime> =
                    postcard::from_bytes_cobs(&mut ser).unwrap();
                let pixel_count = 40;
                next_vm.runtime_mut().init(pixel_count);

                executor_state.with_mut(|executor| {
                    if let Some(executor) = executor {
                        if let Some(vm) = executor.take_vm() {
                            let rt = vm.dismember();
                            *next_vm.runtime_mut() = rt;
                            executor.set_vm(next_vm);
                            executor.start();
                        }
                    } else {
                        let mut nextecutor = Executor::new(next_vm, pixel_count);

                        nextecutor.start();
                        *executor = Some(nextecutor);
                    }
                });
            }
        },
    );

    // TODO this causes a respawn loop, because we depend on executor_state while also modifying it
    let _irish_setter: &UseFuture<()> =
        use_future(&cx, (executor_state,), |(executor_state,)| async move {
            loop {
                executor_state.with_mut(|executor| {
                    if let Some(executor) = executor {
                        executor.do_frame();
                    }
                });
                TimeoutFuture::new(1000).await;
            }
        });

    cx.render(rsx! (
        div {
            style: "text-align: center;",
            h1 { "Yo dawg…" }
            form {
                textarea  {
                    name: "input_js",
                    rows: "20",
                    cols: "80",
                    placeholder: "place code here",
                    value: "{js}",
                    oninput: move |ev| {
                        let val = ev.value.clone();
                        send_to_mcu.send(val.clone());
                        log::debug!("{val}");
                        js.set(val);
                    },
                }
            }
            Pb { }
        }
    ))
}
