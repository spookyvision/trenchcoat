use std::{collections::HashMap, str::from_utf8};

use dioxus::{core::to_owned, prelude::*};
use futures::StreamExt;
use gloo::timers::future::TimeoutFuture;
use trenchcoat::{
    forth::{
        compiler::{compile, Flavor},
        vm::VM,
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

fn main() {
    let base_url: UseState<String>;
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    console_error_panic_hook::set_once();
    dioxus::web::launch(app);
}

#[allow(non_snake_case)]
#[inline_props]
fn Pixel(cx: Scope, executor: UseRef<WebExecutor>) -> Element {
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

fn app(cx: Scope) -> Element {
    let initial_js = include_str!("../../res/rainbow melt.js").to_string();
    let pixel_count = 40;

    let executor = use_ref(&cx, || {
        let mut ser = compile(initial_js.as_str(), Flavor::Pixelblaze).unwrap();
        let mut vm: VM<PixelBlazeFFI, WebRuntime> = postcard::from_bytes_cobs(&mut ser).unwrap();
        vm.runtime_mut().init(pixel_count);
        let mut executor = Executor::new(vm, pixel_count);

        executor.start();
        executor
    });
    let js = use_state(&cx, || initial_js.clone());

    let ex2 = executor.clone();
    let update_executor = use_coroutine(&cx, |mut rx: UnboundedReceiver<Vec<u8>>| async move {
        let executor = ex2;
        while let Some(mut ser) = rx.next().await {
            log::debug!("refresh executor");

            let mut next_vm: VM<PixelBlazeFFI, WebRuntime> =
                postcard::from_bytes_cobs(&mut ser).unwrap();
            next_vm.runtime_mut().init(pixel_count);

            let vm = executor.write_silent().take_vm().unwrap();
            let rt = vm.dismember();
            *next_vm.runtime_mut() = rt;
            executor.write_silent().set_vm(next_vm);
            executor.write().start();
        }
    })
    .to_owned();

    let _code_updated = use_future(&cx, (js,), |(js,)| async move {
        if let Ok(mut ser) = compile(&js, Flavor::Pixelblaze) {
            update_executor.send(ser.clone());

            log::debug!("updating mcu");
            let url = "http://localhost:8008/";
            surf::post(url)
                .content_type("multipart/form-data")
                .body_bytes(&ser)
                .await;
        }
    });

    cx.spawn({
        to_owned![executor];
        async move {
            TimeoutFuture::new(100).await;
            executor.write().do_frame();
        }
    });

    cx.render(rsx! {
        h1 { "Trenchcoat!" }
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
            Pixel { executor: executor.clone() }

    })
}
