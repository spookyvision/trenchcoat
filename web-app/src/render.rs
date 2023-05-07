use chrono::Utc;
use dioxus::prelude::*;
use gloo::timers::future::TimeoutFuture;
use log::debug;
use trenchcoat::{
    forth::vm::CellData,
    pixelblaze::{executor::Executor, ffi::PixelBlazeFFI},
};
use web_sys::InputEvent;

use crate::runtime::WebRuntime;

#[derive(Debug, Clone)]
pub enum RuntimeUi {
    Slider(String),
    Toggle(String),
}

pub const SCALE: f32 = 100.0;
pub fn slider_val_normalized(val: &str) -> f32 {
    val.parse::<f32>().unwrap_or_default() / SCALE
}

pub fn normalized_to_slider_val(val: f32) -> f32 {
    val * SCALE
}

#[allow(non_snake_case)]
#[inline_props]
pub fn UiSlider<'a>(
    cx: Scope<'a>,
    name: &'a str,
    val: f32,
    oninput: EventHandler<'a, FormEvent>,
) -> Element {
    let val = normalized_to_slider_val(*val);
    cx.render(rsx!(
        div {
            input {
                r#type: "range",
                value: "{val}",
                name: "{name}",

                oninput: move |e| oninput.call(e),
            }
            label {
                r#for: "{name}",
                "{name}"
            }
        }

    ))
}
