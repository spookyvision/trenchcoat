use std::time::Instant;

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
pub struct EspRuntime {
    led_idx: usize,
    leds: Option<Vec<Led>>,
    started_at: Instant,
}

impl Default for EspRuntime {
    fn default() -> Self {
        Self {
            led_idx: Default::default(),
            leds: Default::default(),
            started_at: Instant::now(),
        }
    }
}
impl EspRuntime {
    pub fn init(&mut self, pixel_count: usize) {
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
            leds[self.led_idx] = Led::new(h.to_num(), s.to_num(), v.to_num());
            log::info!("ohai {:?}", leds);
            log::warn!("test warn");
            println!("ohai {:?}", leds);
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
        log::debug!("[LOG] {s}");
    }
}
