use std::collections::HashMap;

use dioxus::{prelude::*, web::WebEventExt};
use dioxus_logger::tracing::{error, info, warn, Level};
use dioxus_sdk::utils::channel::{use_channel, use_listen_channel, UseChannel};
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    future, StreamExt,
};
use gloo::timers::future::TimeoutFuture;
use itertools::Itertools;
use render::{slider_val_normalized, RuntimeUi, UiSlider};
use serde::Deserialize;
use trenchcoat::{
    forth::{
        compiler::{compile, Flavor, Source},
        vm::{FuncDef, VM},
    },
    pixelblaze::{executor::Executor, ffi::PixelBlazeFFI},
};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::runtime::WebRuntime;

mod render;
mod runtime;

type WebExecutor = Executor<PixelBlazeFFI, WebRuntime>;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/style.css");

type PBExector = Executor<PixelBlazeFFI, WebRuntime>;
#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
struct AppConfig {
    endpoints: Vec<String>,
    pixel_count: usize,
    initial_js_file: String,
    initial_js: Option<String>,
}

fn main() {
    // for trenchcoat logs. TODO migrate to tracing??
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));

    // sadly DEBUG means swc spam
    dioxus_logger::init(Level::INFO).expect("failed to init logger");

    dioxus::launch(App);
}

fn extract_ui_items(funcs: &HashMap<String, FuncDef<PixelBlazeFFI>>) -> Vec<RuntimeUi> {
    let mut res = vec![];
    for (func_name, _) in funcs.iter().sorted_by(|(k, _), (k2, _)| k.cmp(k2)) {
        if func_name.starts_with("slider") {
            if let Some(label) = func_name.split("slider").nth(1) {
                res.push(RuntimeUi::Slider(label.to_string()));
            }
        } else if func_name.starts_with("toggle") {
            let var = func_name.split("toggle").nth(1);
            error!("todo {func_name}")
        } else if func_name.starts_with("hsvPicker") {
            error!("todo {func_name}")
        } else if func_name.starts_with("rgbPicker") {
            error!("todo {func_name}")
        } else if func_name.starts_with("trigger") {
            error!("todo {func_name}")
        } else if func_name.starts_with("inputNumber") {
            error!("todo {func_name}")
        } else if func_name.starts_with("showNumber") {
            error!("todo {func_name}")
        } else if func_name.starts_with("gauge") {
            error!("todo {func_name}")
        }
    }

    res
}

type SliderData = (String, f32);
#[component]
fn App() -> Element {
    info!("start");

    let config: AppConfig =
        postcard::from_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/config.ser"))).unwrap();

    // TODO why is this an `Option` again?
    let initial_js = config.initial_js.clone().unwrap();

    let pixel_count = config.pixel_count;

    let mut executor = use_signal(|| None);

    let mut ui_items = use_signal(|| vec![]);
    let (sliders_tx, sliders_rx) = mpsc::channel::<(String, f32)>(32);
    let sliders_tx = use_signal(|| sliders_tx);
    let sliders_rx = use_signal(|| sliders_rx);

    let code_updated = use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
        while let Some(code) = rx.next().await {
            info!("code updated");
            match compile(Source::String(code.as_str()), Flavor::Pixelblaze) {
                Ok(mut new_bytecode) => {
                    warn!("TODO send update to endpoints here");
                    // futs.push(async move
                    // future::join_all(futs).await;
                    // bytecode.set(Some(new_bytecode));

                    let mut vm: VM<PixelBlazeFFI, WebRuntime> =
                        postcard::from_bytes_cobs(&mut new_bytecode).unwrap();
                    vm.runtime_mut().init(pixel_count);

                    let funcs = vm.funcs().clone();
                    ui_items.set(extract_ui_items(&funcs));

                    let mut exec = Executor::new(vm, pixel_count);
                    exec.start();
                    executor.set(Some(exec));
                }
                Err(e) => {
                    warn!("compile error {e:?}");
                }
            }
        }
    });
    code_updated.send(initial_js.clone());

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        h1 { "Welcome to Trenchcoat!" }
        form {
            textarea {
                name: "input_js",
                rows: "20",
                cols: "80",
                placeholder: "place code here",
                oninput: move |ev| {
                    let val = ev.value();
                    code_updated.send(val);
                },

                "{initial_js}"
            }
        }
        hr {}
        Trenchcoat {
            executor,
            pixel_count,
            ui_items,
            sliders_tx,
            sliders_rx,
        }
    }
}

#[component]
fn Trenchcoat(
    executor: Signal<Option<WebExecutor>>,
    pixel_count: usize,
    ui_items: Signal<Vec<RuntimeUi>>,
    sliders_tx: Signal<Sender<SliderData>>,
    sliders_rx: Signal<Receiver<SliderData>>,
) -> Element {
    let mut canvas_context: Signal<Option<CanvasRenderingContext2d>> = use_signal(|| None);
    let mut delay = use_signal(|| "50".to_string());

    let exr = executor.read();
    let globals = exr
        .as_ref()
        .map(|ex| ex.globals().cloned())
        .flatten()
        .unwrap_or_default();
    let _r = use_resource(move || async move {
        let Some(mut exec) = executor().clone() else {
            return;
        };

        loop {
            if let Some(context) = canvas_context() {
                // TODO that's not very async of us: it would be better to use StreamExt,
                // but then we'd need an Arc<Mutex<Executor>> and ... bleh. it's fine as is,
                // most time is spent in the executor anyway, not busy waiting the slider channel.
                // it WOULD be nicer though to use mpmc so we can clone a receiver instead of doing try_write
                let Ok(mut sx) = sliders_rx.try_write() else {
                    continue;
                };
                while let Ok(Some((slider_name, slider_value))) = sx.try_next() {
                    exec.on_slider("slider".to_string() + slider_name.as_str(), slider_value);
                }

                if let Err(e) = exec.do_frame() {
                    error!("VM error: {e:?}");
                    return;
                }

                let runtime = exec.runtime().unwrap();
                let leds = runtime.leds().unwrap();

                for (i, led) in leds.iter().enumerate() {
                    let r = led.red * 255.;
                    let g = led.green * 255.;
                    let b = led.blue * 255.;
                    let color = format!("rgb({r},{g},{b})");
                    context.set_fill_style_str(&color);
                    context.fill_rect((i * 4) as f64, 0., 4., 10.);
                }
            }

            TimeoutFuture::new(delay().parse().unwrap()).await;
        }
    });

    let ui_items_comps = ui_items.iter().map(|item| {
        to_owned![item];
        let mut sx = sliders_tx();
        match item {
            RuntimeUi::Slider(name) => {
                let val = globals
                    .get(&name.to_lowercase())
                    .cloned()
                    .flatten()
                    .map(|fv| fv.to_num())
                    .unwrap_or(0.5);
                rsx! {
                    div {
                        UiSlider {
                            key: "{name}",
                            name: name.clone(),
                            val,
                            oninput: {
                                move |ev: FormEvent| {
                                    let val = slider_val_normalized(&ev.value());
                                    if let Err(e) = sx.try_send((name.clone(), val)) {
                                        warn!("slider update error: {e:?}");
                                    }
                                }
                            },
                        }
                    }
                }
            }
            _ => rsx! { "TODO" },
        }
    });

    rsx! {
        {ui_items_comps}
        input {
            r#type: "range",
            min: "16",
            max: "500",
            value: delay,
            oninput: move |ev| {
                delay.set(ev.value());
            },
        }
        label { "frame delay ms: {delay}" }

        canvas {
            id: "pixels",
            width: 400,
            height: 10,
            onmounted: move |ev| {
                if let Some(el) = ev.try_as_web_event() {
                    if let Ok(canvas) = el.dyn_into::<HtmlCanvasElement>() {
                        let context = canvas
                            .get_context("2d")
                            .unwrap()
                            .unwrap()
                            .dyn_into::<web_sys::CanvasRenderingContext2d>()
                            .unwrap();
                        canvas_context.set(Some(context));
                    } else {
                        error!("canvas: could not onmounted");
                    }
                }
            },
        }
    }
}
