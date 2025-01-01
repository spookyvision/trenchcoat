use dioxus::prelude::*;
use dioxus_logger::tracing::warn;

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

#[component]
pub fn UiSlider(name: String, val: f32, oninput: EventHandler<FormEvent>) -> Element {
    let val = normalized_to_slider_val(val);
    warn!("TODO add id");
    rsx! {
        div {
            input {
                r#type: "range",
                value: "{val}",
                name: "{name}",

                oninput,
            }
            label { r#for: "{name}", "{name}" }
        }
    }
}
