use std::{collections::HashMap, time::Duration};

use dioxus::{prelude::*, web::WebEventExt};
use dioxus_logger::tracing::{debug, error, info, warn, Level};
use dioxus_sdk::utils::channel::{use_channel, use_listen_channel, UseChannel};
use futures::{future, StreamExt};
use gloo::timers::future::TimeoutFuture;
use itertools::Itertools;
// use local_subscription::{
//      LocalSubscription, SplitSubscription,
// };
use render::{slider_val_normalized, RuntimeUi, UiSlider};
use serde::Deserialize;
use trenchcoat::{
    forth::{
        compiler::{compile, Flavor, Source},
        util::MockRuntime,
        vm::VM,
    },
    pixelblaze::{executor::Executor, ffi::PixelBlazeFFI, traits::PixelBlazeRuntime},
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

#[component]
fn App() -> Element {
    info!("start");

    let config: AppConfig =
        postcard::from_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/config.ser"))).unwrap();

    // TODO why is this an `Option` again?
    let initial_js = config.initial_js.clone().unwrap();

    let pixel_count = config.pixel_count;

    let mut executor = use_signal(|| None);

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
                    let mut exe = Executor::new(vm, pixel_count);
                    exe.start();
                    executor.set(Some(exe));
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
        Trenchcoat2 { executor, pixel_count }
    }
}

#[component]
fn Trenchcoat2(executor: Signal<Option<WebExecutor>>, pixel_count: usize) -> Element {
    let mut canvas_context: Signal<Option<CanvasRenderingContext2d>> = use_signal(|| None);
    let mut delay = use_signal(|| "50".to_string());
    let _runner = use_future(move || {
        warn!("TODO update slider values");
        async move {
            loop {
                let mut ex = executor();
                if let Some(mut exe) = executor() {
                    if let Some(context) = canvas_context() {
                        exe.do_frame();

                        // while let Ok((name, val)) = slider_rx.try_recv() {
                        //     executor.on_slider("slider".to_string() + &name, val);
                        // }

                        let runtime = exe.runtime().unwrap();
                        let leds = runtime.leds().unwrap();
                        let num_leds = leds.len();

                        for (i, led) in leds.iter().enumerate() {
                            let r = led.red * 255.;
                            let g = led.green * 255.;
                            let b = led.blue * 255.;
                            let color = format!("rgb({r},{g},{b})");
                            context.set_fill_style_str(&color);
                            context.fill_rect((i * 4) as f64, 0., 4., 10.);
                        }
                    }
                }

                TimeoutFuture::new(delay().parse().unwrap()).await;
            }
        }
    });

    rsx! {
        input {
            r#type: "range",
            min: "16",
            max: "500",
            value: delay,
            oninput: move |ev| {
                delay.set(ev.value());
            },
        }
        span { "frame delay ms: {delay}" }

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

#[deprecated]
#[component]
fn Trenchcoat(js: Signal<String>, config: AppConfig) -> Element {
    let pixel_count = config.pixel_count;
    let mut slider_vars = use_signal(|| HashMap::<String, f32>::new());
    let bytecode =
        use_signal(|| compile(Source::String(js.read().as_str()), Flavor::Pixelblaze).unwrap());

    let mut vm = use_signal(|| {
        let mut bytecode = bytecode.read().clone();

        let mut vm: VM<PixelBlazeFFI, WebRuntime> =
            postcard::from_bytes_cobs(&mut bytecode).unwrap();
        vm.runtime_mut().init(pixel_count);
        vm
    });

    let mut executor = use_signal(|| {
        let mut executor = Executor::new(vm.read().clone(), pixel_count);
        executor.start();
        executor
    });

    let slider_tx: UseChannel<(String, f32)> = use_channel(2);

    let slider_rx = use_listen_channel(&slider_tx, move |message| async move {
        warn!("TODO slider {message:?}");
    });

    let ui_items: Signal<Vec<RuntimeUi>> = use_signal(|| vec![]);

    let recompile = use_coroutine(move |mut rx: UnboundedReceiver<Vec<u8>>| {
        to_owned![bytecode];

        async move {
            while let Some(vm_ser) = rx.next().await {
                info!("refresh executor");
                bytecode.set(vm_ser.clone());
            }
        }
    })
    .to_owned();

    // let endpoints = config.endpoints.clone();
    error!("use_future got refactored, no more rerun/dependency mechanism");
    let _code_updated = use_future(move || async move {
        let endpoints = vec!["http://fail"];
        if let Ok(ser) = compile(Source::String(js.read().as_str()), Flavor::Pixelblaze) {
            recompile.send(ser.clone());

            let mut futs = vec![];
            for url in endpoints.iter() {
                let ser = ser.clone();
                futs.push(async move {
                    warn!("TODO update endpoint at {url}");
                    // surf::post(url)
                    //     .content_type("multipart/form-data")
                    //     .body_bytes(&ser)
                    //     .await;
                });
            }
            future::join_all(futs).await;
        }
    });

    let mut canvas_context: Signal<Option<CanvasRenderingContext2d>> = use_signal(|| None);

    let _render_loop = use_future(move || async move {
        to_owned![pixel_count, ui_items, executor, slider_rx, slider_vars];
        async move {
            if let Some(context) = canvas_context.as_ref() {
                let mut bytecode = bytecode.read().clone();

                // TODO we pipe every vm update through a ser+de step
                // because that's how compile() works... suboptimal
                let mut vm: VM<PixelBlazeFFI, WebRuntime> =
                    postcard::from_bytes_cobs(&mut bytecode).unwrap();
                vm.runtime_mut().init(pixel_count);

                let funcs = vm.funcs().clone();
                {
                    let mut exw = executor.write();
                    let old_vm = exw.take_vm().unwrap();
                    let rt = old_vm.dismember();
                    *vm.runtime_mut() = rt;
                    exw.set_vm(vm);
                    exw.start();
                }

                let mut next_ui_items: Vec<RuntimeUi> = vec![];
                for (func_name, _) in funcs.iter().sorted_by(|(k, _), (k2, _)| k.cmp(k2)) {
                    if func_name.starts_with("slider") {
                        if let Some(label) = func_name.split("slider").nth(1) {
                            next_ui_items.push(RuntimeUi::Slider(label.to_string()));
                            slider_vars.with(|slider_vars| {
                                if let Some(val) = slider_vars.get(label) {
                                    executor
                                        .write()
                                        .on_slider("slider".to_string() + label, *val);
                                }
                            })
                        }
                    } else if func_name.starts_with("toggle") {
                        let var = func_name.split("toggle").nth(1);
                    } else if func_name.starts_with("hsvPicker") {
                        log::error!("todo")
                    } else if func_name.starts_with("rgbPicker") {
                        log::error!("todo")
                    } else if func_name.starts_with("trigger") {
                        log::error!("todo")
                    } else if func_name.starts_with("inputNumber") {
                        log::error!("todo")
                    } else if func_name.starts_with("showNumber") {
                        log::error!("todo")
                    } else if func_name.starts_with("gauge") {
                        log::error!("todo")
                    }
                }

                ui_items.set(next_ui_items);

                loop {
                    let mut exw = executor.write();
                    warn!("TODO update slider values");
                    // while let Ok((name, val)) = slider_rx.try_recv() {
                    //     executor.on_slider("slider".to_string() + &name, val);
                    // }
                    exw.do_frame();
                    let runtime = exw.runtime().unwrap();
                    let leds = runtime.leds().unwrap();
                    let num_leds = leds.len();

                    for (i, led) in leds.iter().enumerate() {
                        let r = led.red * 255.;
                        let g = led.green * 255.;
                        let b = led.blue * 255.;
                        let color = format!("rgb({r},{g},{b})");
                        warn!("deprecated: set_fill_style");
                        context.set_fill_style(&color.into());
                        context.fill_rect((i * 4) as f64, 0., 4., 10.);
                    }

                    TimeoutFuture::new(30).await;
                }
            }
        }
    });

    let rendered_items = ui_items.iter().map(|item| match &*item {
        RuntimeUi::Slider(name) => rsx!(
            div {
                UiSlider {
                    key: "{name}",
                    name,
                    val: slider_vars
                        .with(|slider_vars| {
                            let res = slider_vars.get(name).cloned().unwrap_or_default();
                            res
                        }),
                    oninput: move |ev: FormEvent| {
                        let val = slider_val_normalized(&ev.value());
                        /// slider_vars.with_mut(|slider_vars| slider_vars.insert(name.clone(), val));
                        /// slider_tx.send((name.clone(), val));
                        warn!("TODO oninput");
                    },
                }
            }
        ),
        _ => rsx! { "TODO" },
    });

    rsx! {
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
        {rendered_items}
    }
}
