use chrono::Utc;
use dioxus::{core::to_owned, prelude::*};
use fermi::{use_atom_state, use_read, Atom, AtomState};
use gloo::timers::future::TimeoutFuture;
use log::debug;

use crate::runtime::Led;

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
