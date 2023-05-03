use chrono::Utc;
use dioxus::prelude::*;
use fermi::{use_atom_state, use_read, Atom, AtomState};
use gloo::timers::future::TimeoutFuture;
use log::debug;
use trenchcoat::{
    forth::vm::CellData,
    pixelblaze::{executor::Executor, ffi::PixelBlazeFFI},
};

use crate::{
    runtime::{Led, WebRuntime},
    SliderVars,
};

pub(crate) enum RuntimeUi {
    Slider(String),
    Toggle(String),
}

#[allow(non_snake_case)]
#[inline_props]
pub(crate) fn UiSlider(
    cx: Scope,
    name: String,
    vars: UseState<SliderVars>,
    executor: UseRef<Executor<PixelBlazeFFI, WebRuntime>>,
) -> Element {
    const SCALE: f32 = 100.0;

    let val = vars.get().get(name.as_str()).cloned().unwrap_or_default();
    cx.render(rsx!(
        input {
            r#type: "range",
            value: "{val * SCALE}",
            name: "{name}",

            oninput: move |ev| {
                let new_val = ev.value.parse::<f32>().unwrap_or_default() / SCALE;
                // TODO what exactly is the downside of using write?
                //executor.write().on_slider(...)
                vars.with_mut(|vars| {vars.insert(name.clone(), new_val); });
                executor.with_mut(|ex| ex.on_slider("slider".to_string() + &name, new_val));
            },
        }
        label {
            r#for: "{name}",
            "{name}"
        }
    ))
}

#[allow(non_snake_case)]
#[inline_props]
pub(crate) fn LedWidget(cx: Scope, led: Led) -> Element {
    cx.render(rsx!(div {
        class: "square",
        style: format_args!(
            "background-color: hsl({}turn,{}%,{}%)",
            led.h,
            led.s * 100.,
            led.l * 100.
        ),
    }))
}
