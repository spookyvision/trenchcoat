use chrono::{DateTime, Utc};
use trenchcoat::{
    forth::vm::CellData, pixelblaze::traits::Peripherals, vanillajs::runtime::VanillaJSRuntime,
};

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

#[derive(Clone, Debug, PartialEq)]
pub struct WebRuntime {
    led_idx: usize,
    leds: Vec<Led>,
    started_at: DateTime<Utc>,
}

impl WebRuntime {
    pub fn new(pixel_count: usize) -> Self {
        let mut leds = Vec::with_capacity(pixel_count);
        for _ in 0..pixel_count {
            leds.push(Led::default())
        }
        Self {
            led_idx: 0,
            leds,
            started_at: Utc::now(),
        }
    }

    pub fn leds(&self) -> &[Led] {
        &self.leds
    }
}

impl Peripherals for WebRuntime {
    fn led_hsv(&mut self, h: CellData, s: CellData, v: CellData) {
        self.leds[self.led_idx] = Led::new(h.to_num(), s.to_num(), v.to_num());
    }

    fn set_led_idx(&mut self, idx: usize) {
        self.led_idx = idx;
    }
}

impl VanillaJSRuntime for WebRuntime {
    fn time_millis(&mut self) -> u32 {
        let dt = Utc::now().signed_duration_since(self.started_at);
        dt.num_milliseconds() as u32
    }

    fn log(&mut self, s: &str) {
        log::debug!("[LOG] {s}");
    }
}
