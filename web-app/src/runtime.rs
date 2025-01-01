use chrono::{DateTime, Utc};
use palette::{hsl, okhsl::Okhsl, FromColor, Hsl, Hsv, Srgb};
use trenchcoat::{
    forth::vm::CellData, pixelblaze::traits::Peripherals, vanillajs::runtime::VanillaJSRuntime,
};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct WebRuntime {
    led_idx: usize,
    leds: Option<Vec<Srgb>>,
    // TODO default gives 1970, not exactly a true "started_at"
    started_at: DateTime<Utc>,
}

impl WebRuntime {
    // TODO this init business kinda stinks, should have a proper constructor
    pub fn init(&mut self, pixel_count: usize) {
        let mut leds = Vec::with_capacity(pixel_count);
        for _ in 0..pixel_count {
            leds.push(Srgb::default())
        }
        self.leds = Some(leds);
    }
    pub fn leds(&self) -> Option<&Vec<Srgb>> {
        self.leds.as_ref()
    }
}

impl Peripherals for WebRuntime {
    fn led_hsv(&mut self, h: CellData, s: CellData, v: CellData) {
        if let Some(leds) = self.leds.as_mut() {
            let h: f32 = h.to_num();
            let s: f32 = s.to_num();
            let v: f32 = v.to_num();
            let hsv = Hsv::new(h * 360., s, v);
            leds[self.led_idx] = Srgb::from_color(hsv);
        }
    }

    fn led_rgb(&mut self, r: CellData, g: CellData, b: CellData) {
        if let Some(leds) = self.leds.as_mut() {
            let r: f32 = r.to_num();
            let g: f32 = g.to_num();
            let b: f32 = b.to_num();
            leds[self.led_idx] = Srgb::new(r, g, b);
        }
    }

    fn ext_led_okhsl(&mut self, h: CellData, s: CellData, l: CellData) {
        if let Some(leds) = self.leds.as_mut() {
            let h: f32 = h.to_num();
            let s: f32 = s.to_num();
            let l: f32 = l.to_num();
            let okhsl = Okhsl::new(h * 360., s, l);
            leds[self.led_idx] = Srgb::from_color(okhsl);
        }
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
