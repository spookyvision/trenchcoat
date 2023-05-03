use std::{collections::HashMap, str::from_utf8};

use config::Config;
use dioxus::prelude::*;
use futures::{future, StreamExt};
use gloo::timers::future::TimeoutFuture;
use itertools::Itertools;
use log::debug;
use render::{RuntimeUi, UiSlider};
use serde::Deserialize;
use trenchcoat::{
    forth::{
        compiler::{compile, Flavor},
        vm::{CellData, VM},
    },
    pixelblaze::{executor::Executor, ffi::PixelBlazeFFI, traits::PixelBlazeRuntime},
};

use crate::{
    render::LedWidget,
    runtime::{Led, WebRuntime},
};

mod render;
mod runtime;

type WebExecutor = Executor<PixelBlazeFFI, WebRuntime>;
struct RecompileState {
    vm_bytes: Vec<u8>,
    slider_vars: SliderVars,
}

pub(crate) type SliderVars = im_rc::HashMap<String, f32>;

fn main() {
    console_error_panic_hook::set_once();
    debug!("?");
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));

    dioxus_web::launch_with_props(app, (), dioxus_web::Config::new());
}

#[allow(non_snake_case)]
#[inline_props]
fn Pixels(cx: Scope, executor: UseRef<WebExecutor>) -> Element {
    let executor = executor.read();
    let runtime = executor.runtime().unwrap();
    let leds = runtime.leds().unwrap();
    let inner = leds.iter().cloned().enumerate().map(|(led_id, led)| {
        rsx! {
            div {
                class: "square-container",
                key: "led-{led_id}",
                LedWidget { led: led }
            }
        }
    });
    let content = rsx!(div { inner });

    cx.render(content)
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct AppConfig {
    endpoints: Vec<String>,
    pixel_count: usize,
    initial_js_file: String,
    initial_js: Option<String>,
}

#[allow(non_snake_case)]
#[inline_props]
fn RenderExecutor(cx: Scope, js: UseState<String>, config: AppConfig) -> Element {
    let pixel_count = config.pixel_count;
    // TODO add manual flush?
    // TODO remove im collections; proper usage: see embedded ui
    let slider_vars = use_state(&cx, SliderVars::default);

    let ui_items = use_state(cx, || {
        let res: Vec<RuntimeUi> = vec![];
        res
    });

    let executor: &UseRef<Executor<PixelBlazeFFI, WebRuntime>> = use_ref(&cx, || {
        let mut ser = compile(js.as_str(), Flavor::Pixelblaze).unwrap();
        let mut vm: VM<PixelBlazeFFI, WebRuntime> = postcard::from_bytes_cobs(&mut ser).unwrap();
        vm.runtime_mut().init(pixel_count);
        let mut executor = Executor::new(vm, pixel_count);

        executor.start();
        executor
    });

    cx.spawn({
        to_owned![executor];
        async move {
            TimeoutFuture::new(100).await;
            executor.write().do_frame();
        }
    });

    let recompile = use_coroutine(&cx, |mut rx: UnboundedReceiver<RecompileState>| {
        to_owned![executor, ui_items];

        async move {
            while let Some(mut recompile_state) = rx.next().await {
                debug!("refresh executor");

                let RecompileState {
                    mut vm_bytes,
                    slider_vars,
                } = recompile_state;
                let mut next_vm: VM<PixelBlazeFFI, WebRuntime> =
                    postcard::from_bytes_cobs(&mut vm_bytes).unwrap();
                next_vm.runtime_mut().init(pixel_count);

                let mut next_ui_items = vec![];
                for (func_name, _) in next_vm
                    .funcs()
                    .iter()
                    .sorted_by(|(k, _), (k2, _)| k.cmp(k2))
                {
                    if func_name.starts_with("slider") {
                        if let Some(label) = func_name.split("slider").nth(1) {
                            next_ui_items.push(RuntimeUi::Slider(label.to_string()))
                        }
                    } else if func_name.starts_with("toggle") {
                        let var = func_name.split("toggle").nth(1);
                        debug!("{var:?}");
                    } else if func_name.starts_with("hsvPicker") {
                        todo!()
                    } else if func_name.starts_with("rgbPicker") {
                        todo!()
                    } else if func_name.starts_with("trigger") {
                        todo!()
                    } else if func_name.starts_with("inputNumber") {
                        todo!()
                    } else if func_name.starts_with("showNumber") {
                        todo!()
                    } else if func_name.starts_with("gauge") {
                        todo!()
                    }
                }

                ui_items.set(next_ui_items);

                let vm = executor.write_silent().take_vm().unwrap();
                let rt = vm.dismember();
                *next_vm.runtime_mut() = rt;
                executor.write_silent().set_vm(next_vm);
                executor.write().start();

                for (k, v) in slider_vars.iter() {
                    executor.with_mut(|ex| ex.on_slider("slider".to_string() + k, *v));
                }
            }
        }
    })
    .to_owned();
    let endpoints = config.endpoints.clone();

    let _code_updated = use_future(&cx, (js, slider_vars), |(js, slider_vars)| async move {
        if let Ok(mut ser) = compile(&js, Flavor::Pixelblaze) {
            let state = RecompileState {
                vm_bytes: ser.clone(),
                slider_vars: slider_vars.get().clone(),
            };
            recompile.send(state);

            let mut futs = vec![];
            for url in endpoints.iter().cloned() {
                let ser = ser.clone();
                futs.push(async move {
                    debug!("updating endpoint at {url}");
                    surf::post(url)
                        .content_type("multipart/form-data")
                        .body_bytes(&ser)
                        .await;
                });
            }
            future::join_all(futs).await;
        }
    });

    cx.render(rsx! {

        Pixels { executor: executor.clone() }

        ui_items.iter().map(|item| match item {
            RuntimeUi::Slider(name) => rsx!(
                UiSlider {
                    key: "{name}",
                    name: name.clone(),
                    vars: slider_vars.clone(),
                    executor: executor.clone() }
            ),
            _ => rsx!{"TODO"}

        })
    })
}

fn app(cx: Scope) -> Element {
    log::info!("start");

    let config: AppConfig =
        postcard::from_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/config.ser"))).unwrap();

    // TODO why is this an `Option` again?
    let initial_js = config.initial_js.clone().unwrap();

    let js = use_state(&cx, || initial_js.clone());

    cx.render(rsx! {
        h1 { "Welcome to Trenchcoat!" }
        form {
            textarea  {
                name: "input_js",
                rows: "20",
                cols: "80",
                placeholder: "place code here",
                oninput: move |ev| {
                    let val = ev.value.clone();
                    js.set(val);
                },

                "{initial_js}"
            }
        }
        hr {}
        RenderExecutor { js: js.clone(), config: config }

    })
}
