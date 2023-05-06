use std::{collections::HashMap, str::from_utf8};

use config::Config;
use dioxus::prelude::*;
use futures::{channel::oneshot, future, StreamExt};
use gloo::{timers::future::TimeoutFuture, utils::window};
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
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

use crate::runtime::WebRuntime;

mod render;
mod runtime;

type WebExecutor = Executor<PixelBlazeFFI, WebRuntime>;
struct RecompileState {
    vm_bytes: Vec<u8>,
    slider_vars: SliderVars,
}

pub(crate) type SliderVars = im_rc::HashMap<String, f32>;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct AppConfig {
    endpoints: Vec<String>,
    pixel_count: usize,
    initial_js_file: String,
    initial_js: Option<String>,
}

fn main() {
    console_error_panic_hook::set_once();
    debug!("?");
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));

    dioxus_web::launch_with_props(app, (), dioxus_web::Config::new());
}

#[allow(non_snake_case)]
#[inline_props]
fn Pixels(cx: Scope, bytecode: UseState<Vec<u8>>, pixel_count: usize) -> Element {
    let canvas_context: &UseState<Option<CanvasRenderingContext2d>> = use_state(&cx, || None);

    let render = use_future(
        cx,
        (bytecode, canvas_context),
        |(bytecode, canvas_context)| {
            to_owned![pixel_count];
            async move {
                if let Some(context) = canvas_context.get() {
                    let mut bytecode = bytecode.get().clone();

                    let mut vm: VM<PixelBlazeFFI, WebRuntime> =
                        postcard::from_bytes_cobs(&mut bytecode).unwrap();
                    vm.runtime_mut().init(pixel_count);

                    let mut next_ui_items = vec![];
                    for (func_name, _) in vm.funcs().iter().sorted_by(|(k, _), (k2, _)| k.cmp(k2)) {
                        if func_name.starts_with("slider") {
                            if let Some(label) = func_name.split("slider").nth(1) {
                                next_ui_items.push(RuntimeUi::Slider(label.to_string()))
                            }
                        } else if func_name.starts_with("toggle") {
                            let var = func_name.split("toggle").nth(1);
                            debug!("{var:?}");
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

                    let mut executor = Executor::new(vm, pixel_count);

                    executor.start();
                    // TODO
                    // executor.with_mut(|ex| ex.on_slider("slider".to_string() + &name, new_val));

                    loop {
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

    cx.render(rsx!(canvas { id: "pixels" }))
}

#[allow(non_snake_case)]
#[inline_props]
fn RenderExecutor(cx: Scope, js: UseState<String>, config: AppConfig) -> Element {
    let pixel_count = config.pixel_count;
    // TODO add manual flush?
    // TODO remove im collections; proper usage: see embedded ui
    let slider_vars = use_state(&cx, SliderVars::default);

    let bytecode = use_state(&cx, || compile(&js, Flavor::Pixelblaze).unwrap());
    let ui_items = use_ref(cx, || {
        let res: Vec<RuntimeUi> = vec![];
        res
    });

    let recompile = use_coroutine(&cx, |mut rx: UnboundedReceiver<Vec<u8>>| {
        to_owned![ui_items, js, bytecode];

        async move {
            while let Some(mut vm_ser) = rx.next().await {
                debug!("refresh executor");
                bytecode.set(vm_ser.clone());
            }
        }
    })
    .to_owned();
    let endpoints = config.endpoints.clone();

    let _vars_updates = use_future(&cx, slider_vars, |slider_vars| async move {
        debug!("slider vars updated");
    });
    let _code_updated = use_future(&cx, js, |js| async move {
        if let Ok(mut ser) = compile(&js, Flavor::Pixelblaze) {
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

    cx.render(rsx! {

        Pixels { bytecode: bytecode.clone(), pixel_count: pixel_count }

        ui_items.read().iter().map(|item| match item {
            RuntimeUi::Slider(name) => rsx!(
                UiSlider {
                    key: "{name}",
                    name: name.clone(),
                    vars: slider_vars.clone(),
                    }
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
