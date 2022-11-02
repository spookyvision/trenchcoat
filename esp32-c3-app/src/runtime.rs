use std::time::Instant;

use log::{debug, info, warn};
use rgb::RGB8;
use trenchcoat::{
    forth::vm::CellData, pixelblaze::traits::Peripherals, vanillajs::runtime::VanillaJSRuntime,
};

use crate::bsc::led::WS2812RMT;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Led {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

impl Led {
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self { h, s, l }
    }
}

pub struct EspRuntime {
    led_peri: Option<WS2812RMT>,
    led_idx: usize,
    leds: Option<Vec<Led>>,
    started_at: Instant,
}

impl Default for EspRuntime {
    fn default() -> Self {
        Self {
            led_peri: None,
            led_idx: Default::default(),
            leds: Default::default(),
            started_at: Instant::now(),
        }
    }
}
impl EspRuntime {
    pub fn init(&mut self, pixel_count: usize) {
        let mut led_peri = WS2812RMT::new().expect("could not initialize LED peripheral");
        led_peri
            .set_pixel(RGB8::new(0, 1, 1))
            .expect("could not set pixel");
        self.led_peri = Some(led_peri);
        let mut leds = Vec::with_capacity(pixel_count);
        for _ in 0..pixel_count {
            leds.push(Led::default())
        }
        self.leds = Some(leds);
    }
    pub fn leds(&self) -> Option<&Vec<Led>> {
        self.leds.as_ref()
    }
}

impl Peripherals for EspRuntime {
    fn led_hsv(&mut self, h: CellData, s: CellData, v: CellData) {
        if let Some(leds) = self.leds.as_mut() {
            let h: u8 = (h * 255).to_num();
            let s: u8 = (s * 255).to_num();
            let v: u8 = (v * 255).to_num();
            let rgb = hsv2rgb(h, s, v);

            if let Some(led_peri) = self.led_peri.as_mut() {
                debug!("{rgb:?}");
                led_peri.set_pixel(rgb);
            }
        }
    }

    fn set_led_idx(&mut self, idx: usize) {
        self.led_idx = idx;
    }
}

impl VanillaJSRuntime for EspRuntime {
    fn time_millis(&mut self) -> u32 {
        self.started_at.elapsed().as_millis() as u32
    }

    fn log(&mut self, s: &str) {
        debug!("[LOG] {s}");
    }
}

use rgb::RGB;
pub fn hsv2rgb(h: u8, s: u8, v: u8) -> RGB8 {
    let v = h as u16;
    let s = s as u16;
    let f = (h as u16 * 2 % 85) * 3; // relative interval

    let p = v * (255 - s) / 255;
    let q = v * (255 - (s * f) / 255) / 255;
    let t = v * (255 - (s * (255 - f)) / 255) / 255;
    match h {
        0..=42 => RGB {
            r: v as u8,
            g: t as u8,
            b: p as u8,
        },
        43..=84 => RGB {
            r: q as u8,
            g: v as u8,
            b: p as u8,
        },
        85..=127 => RGB {
            r: p as u8,
            g: v as u8,
            b: t as u8,
        },
        128..=169 => RGB {
            r: p as u8,
            g: q as u8,
            b: v as u8,
        },
        170..=212 => RGB {
            r: t as u8,
            g: p as u8,
            b: v as u8,
        },
        213..=254 => RGB {
            r: v as u8,
            g: p as u8,
            b: q as u8,
        },
        255 => RGB {
            r: v as u8,
            g: t as u8,
            b: p as u8,
        },
    }
}
