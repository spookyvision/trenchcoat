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

#[derive(Clone, Debug, PartialEq, Default)]
pub struct WebRuntime {
    led_idx: usize,
    leds: Option<Vec<Led>>,
    // TODO default gives 1970, not exactly a true "started_at"
    started_at: DateTime<Utc>,
}

impl WebRuntime {
    // TODO this init business kinda stinks, should have a proper constructor
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

fn hsv2hsl(h: f32, mut s: f32, v: f32) -> (f32, f32, f32) {
    let l = (2. - s) * v / 2.;

    if (l != 0.) {
        if (l == 1.) {
            s = 0.;
        } else if (l < 0.5) {
            s = s * v / (l * 2.);
        } else {
            s = s * v / (2. - l * 2.);
        }
    }

    (h, s, l)
}
impl Peripherals for WebRuntime {
    fn led_hsv(&mut self, h: CellData, s: CellData, v: CellData) {
        if let Some(leds) = self.leds.as_mut() {
            // CSS uses HSL -> convert HSV to HSL
            let (h, s, l) = hsv2hsl(h.to_num(), s.to_num(), v.to_num());
            leds[self.led_idx] = Led::new(h, s, l);
        }
    }

    fn led_rgb(&mut self, r: CellData, g: CellData, b: CellData) {
        use palette::{FromColor, Hsl, Srgb};
        if let Some(leds) = self.leds.as_mut() {
            let rgb = Srgb::from_components((r.to_num(), g.to_num(), b.to_num()));
            let hsv = Hsl::from_color(rgb);
            let h = hsv.hue.into_positive_degrees() / 360.;
            let s = hsv.saturation;
            let l = hsv.lightness;
            leds[self.led_idx] = Led::new(h, s, l);
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
