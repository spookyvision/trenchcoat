use std::time::Instant;

#[cfg(feature = "apa102")]
use espidf_apa102::Apa;
use log::{debug, info, warn};
use rgb::RGB8;
use trenchcoat::{
    forth::vm::CellData, pixelblaze::traits::Peripherals, vanillajs::runtime::VanillaJSRuntime,
};

use crate::app_config::AppConfig;
#[cfg(feature = "ws2812")]
use crate::ws_peri::Peri;

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
    #[cfg(feature = "apa102")]
    led_peri: Option<Apa>,
    #[cfg(all(feature = "ws2812", not(feature = "apa102")))]
    led_peri: Option<Peri>,
    led_idx: usize,
    leds: Option<Vec<RGB8>>,
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
    pub(crate) fn init(&mut self, config: &AppConfig) {
        log::info!("RT init");
        log::debug!("config {config:?}");
        #[cfg(feature = "ws2812")]
        let led_peri = Peri::new(config.data_pin, config.pixel_count);
        #[cfg(feature = "apa102")]
        let led_peri = Apa::new(espidf_apa102::Config::new(
            config.data_pin,
            config.clock_pin.unwrap(),
        ));

        self.led_peri = Some(led_peri);
        log::info!("LED peripheral ok");
        let mut leds = Vec::with_capacity(config.pixel_count);
        for _ in 0..config.pixel_count {
            leds.push(RGB8::default())
        }
        self.leds = Some(leds);
    }
}

impl Peripherals for EspRuntime {
    fn led_rgb(&mut self, r: CellData, g: CellData, b: CellData) {
        if let Some(leds) = self.leds.as_mut() {
            let rgb = rgb::RGB8::new(r.to_num(), g.to_num(), b.to_num());
            leds[self.led_idx] = rgb;
            if let Some(led_peri) = self.led_peri.as_mut() {
                // TODO wart
                #[cfg(all(feature = "ws2812", not(feature = "apa102")))]
                led_peri.set_rgb(self.led_idx, rgb);
                #[cfg(all(feature = "apa102", not(feature = "ws2812")))]
                led_peri.set_pixel(self.led_idx, rgb.into());
            }
        }
    }
    fn led_hsv(&mut self, h: CellData, s: CellData, v: CellData) {
        if let Some(leds) = self.leds.as_mut() {
            let h: u8 = (h * 255).to_num();
            let s: u8 = (s * 255).to_num();
            let v: u8 = (v * 255).to_num();
            let rgb = hsv2rgb(h, s, v);

            leds[self.led_idx] = rgb;
            if let Some(led_peri) = self.led_peri.as_mut() {
                // TODO wart
                #[cfg(all(feature = "ws2812", not(feature = "apa102")))]
                led_peri.set_rgb(self.led_idx, rgb);
                #[cfg(all(feature = "apa102", not(feature = "ws2812")))]
                led_peri.set_pixel(self.led_idx, rgb.into());
            }
        }
    }

    fn set_led_idx(&mut self, idx: usize) {
        self.led_idx = idx;
    }

    fn led_commit(&mut self) {
        if let Some(led_peri) = self.led_peri.as_mut() {
            // log::trace!("flush");
            led_peri.flush();
        }
    }
}

impl VanillaJSRuntime for EspRuntime {
    fn time_millis(&mut self) -> u32 {
        self.started_at.elapsed().as_millis() as u32
    }

    fn log(&mut self, s: &str) {
        // debug!("[LOG] {s}");
    }
}

use rgb::RGB;
pub fn hsv2rgb(h: u8, s: u8, v: u8) -> RGB8 {
    let v = v as u16;
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
