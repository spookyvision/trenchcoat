use std::{cell::RefCell, collections::HashMap, rc::Rc, str::from_utf8, sync::mpsc};

use config::Config;
use dioxus::prelude::*;
use futures::{channel::oneshot, future, StreamExt};
use gloo::{timers::future::TimeoutFuture, utils::window};
use itertools::Itertools;
use local_subscription::{
    use_local_subscription_root, use_split_subscriptions, LocalSubscription, SplitSubscription,
};
use log::{debug, warn};
use render::{slider_val_normalized, RuntimeUi, UiSlider};
use serde::Deserialize;
use trenchcoat::{
    forth::{
        compiler::{compile, Flavor, Source},
        vm::{CellData, VM},
    },
    pixelblaze::{executor::Executor, ffi::PixelBlazeFFI, traits::PixelBlazeRuntime},
};
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

use crate::runtime::WebRuntime;

mod local_subscription;
mod render;
mod runtime;

type WebExecutor = Executor<PixelBlazeFFI, WebRuntime>;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct AppConfig {
    endpoints: Vec<String>,
    pixel_count: usize,
    initial_js_file: String,
    initial_js: Option<String>,
}

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));

    dioxus_web::launch_with_props(app, (), dioxus_web::Config::new());
}

type PBExector = Executor<PixelBlazeFFI, WebRuntime>;
type VMState = SplitSubscription<Vec<RuntimeUi>, PBExector>;

#[allow(non_snake_case)]
#[inline_props]
fn Trenchcoat(cx: Scope, js: UseState<String>, config: AppConfig) -> Element {
    let pixel_count = config.pixel_count;
    let slider_vars = use_ref(&cx, || HashMap::<String, f32>::new());
    let bytecode = use_state(&cx, || {
        compile(Source::String(&js), Flavor::Pixelblaze).unwrap()
    });

    let (slider_tx, slider_rx) = cx.use_hook(|| {
        let (tx, rx) = std::sync::mpsc::channel::<(String, f32)>();
        (tx, Rc::new(rx))
    });

    let vm_state: &VMState = use_context_provider(cx, || {
        let mut bytecode = bytecode.get().clone();

        let mut vm: VM<PixelBlazeFFI, WebRuntime> =
            postcard::from_bytes_cobs(&mut bytecode).unwrap();
        vm.runtime_mut().init(pixel_count);

        let mut executor = Executor::new(vm, pixel_count);
        executor.start();

        let state = LocalSubscription::create(cx, Default::default());
        SplitSubscription::new(state, executor)
    });

    let executor = vm_state.t.clone();

    let ui_items: &UseState<Vec<RuntimeUi>> = use_state(cx, || vec![]);

    let recompile = use_coroutine(&cx, |mut rx: UnboundedReceiver<Vec<u8>>| {
        to_owned![bytecode];

        async move {
            while let Some(mut vm_ser) = rx.next().await {
                debug!("refresh executor");
                bytecode.set(vm_ser.clone());
            }
        }
    })
    .to_owned();
    let endpoints = config.endpoints.clone();

    let _code_updated = use_future(&cx, js, |js| async move {
        if let Ok(mut ser) = compile(Source::String(&js), Flavor::Pixelblaze) {
            recompile.send(ser.clone());

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

    let canvas_context: &UseState<Option<CanvasRenderingContext2d>> = use_state(&cx, || None);

    let render = use_future(
        cx,
        (bytecode, canvas_context),
        |(bytecode, canvas_context)| {
            to_owned![pixel_count, ui_items, executor, slider_rx, slider_vars];
            let c = canvas_context.get();
            async move {
                if let Some(context) = canvas_context.get() {
                    let mut bytecode = bytecode.get().clone();

                    // TODO we pipe every vm update through a ser+de step
                    // because that's how compile() works... suboptimal
                    let mut vm: VM<PixelBlazeFFI, WebRuntime> =
                        postcard::from_bytes_cobs(&mut bytecode).unwrap();
                    vm.runtime_mut().init(pixel_count);

                    let funcs = vm.funcs().clone();
                    {
                        let mut executor = executor.lock().unwrap();
                        let old_vm = executor.take_vm().unwrap();
                        let rt = old_vm.dismember();
                        *vm.runtime_mut() = rt;
                        executor.set_vm(vm);
                        executor.start();
                    }

                    let mut next_ui_items: Vec<RuntimeUi> = vec![];
                    for (func_name, _) in funcs.iter().sorted_by(|(k, _), (k2, _)| k.cmp(k2)) {
                        if func_name.starts_with("slider") {
                            if let Some(label) = func_name.split("slider").nth(1) {
                                next_ui_items.push(RuntimeUi::Slider(label.to_string()));
                                slider_vars.with(|slider_vars| {
                                    if let Some(val) = slider_vars.get(label) {
                                        executor
                                            .lock()
                                            .unwrap()
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
                        if let Ok(mut executor) = executor.lock() {
                            while let Ok((name, val)) = slider_rx.try_recv() {
                                executor.on_slider("slider".to_string() + &name, val);
                            }
                            executor.do_frame();
                            let runtime = executor.runtime().unwrap();
                            let leds = runtime.leds().unwrap();
                            let num_leds = leds.len();

                            for (i, led) in leds.iter().enumerate() {
                                let r = led.red * 255.;
                                let g = led.green * 255.;
                                let b = led.blue * 255.;
                                let color = format!("rgb({r},{g},{b})");
                                context.set_fill_style(&color.into());
                                context.fill_rect((i * 4) as f64, 0., 4., 10.);
                            }
                        }

                        TimeoutFuture::new(30).await;
                    }
                }
            }
        },
    );

    use_effect(cx, (), |_| {
        to_owned![canvas_context];
        async move {
            let document = window().document();
            let canvas = document.unwrap().get_element_by_id("pixels").unwrap();
            let canvas: web_sys::HtmlCanvasElement = canvas
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .map_err(|_| ())
                .unwrap();
            let context = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();
            canvas_context.set(Some(context));
        }
    });

    cx.render(rsx! {
        canvas {
            id: "pixels",
            width: 400,
            height: 10
        }
        ui_items.iter().map(|item| {
            to_owned![slider_tx];
            match item {
                RuntimeUi::Slider(name) => rsx!(
                    div {
                        UiSlider {
                            key: "{name}",
                            name: name,
                            val: slider_vars.with(|slider_vars| {
                                let res = slider_vars.get(name).cloned().unwrap_or_default();
                                res
                            }),
                            oninput: move |ev: FormEvent| {
                                let val = slider_val_normalized(&ev.value);
                                slider_vars.with_mut(|slider_vars| slider_vars.insert(name.clone(), val));
                                slider_tx.send((name.clone(), val));
                            },
                        }
                    }

                ),
                _ => rsx!{"TODO"}

            }
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
        Trenchcoat { js: js.clone(), config: config }

    })
}
