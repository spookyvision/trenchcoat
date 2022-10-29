use f4_peri::ws2812::WS;
use smart_leds::{SmartLedsWrite, RGB8};
use trenchcoat::{
    forth::vm::CellData, pixelblaze::traits::Peripherals, vanillajs::runtime::VanillaJSRuntime,
};

#[derive(Default)]
pub struct F4Runtime {
    led_idx: usize,
    time: u32,
    leds: [RGB8; 16],
    ws: Option<WS>,
}

impl PartialEq for F4Runtime {
    fn eq(&self, other: &Self) -> bool {
        self.led_idx == other.led_idx && self.time == other.time
    }
}

impl Eq for F4Runtime {}

impl F4Runtime {
    pub fn new(ws: WS) -> Self {
        Self {
            led_idx: 0,
            time: 0,
            leds: Default::default(),
            ws: Some(ws),
        }
    }

    pub fn step_ms(&mut self, dt: i32) {
        self.time = self.time.wrapping_add_signed(dt);
    }

    pub fn set_ws(&mut self, ws: Option<WS>) {
        self.ws = ws;
    }

    pub fn leds_mut(&mut self) -> &mut [RGB8; 16] {
        &mut self.leds
    }
}

impl Peripherals for F4Runtime {
    fn set_led_idx(&mut self, idx: usize) {
        self.led_idx = idx;
    }

    fn led_hsv(&mut self, h: CellData, s: CellData, v: CellData) {
        // defmt::debug!("LED[{}] HSV({},{},{})", self.led_idx, h, s, v);
        self.leds[self.led_idx] = hsv2rgb(h, s, v);
    }

    fn led_begin(&mut self) {}

    fn led_commit(&mut self) {
        if let Some(ws) = self.ws.as_mut() {
            ws.write(self.leds.iter().cloned()).unwrap();
        }
    }
}

impl VanillaJSRuntime for F4Runtime {
    fn time_millis(&mut self) -> u32 {
        self.time
    }

    fn log(&mut self, s: &str) {
        defmt::debug!("{}", s);
    }
}

pub fn hsv2rgb(h: CellData, s: CellData, v: CellData) -> RGB8 {
    let h: u16 = (h * 255).to_num();
    let s: u16 = (s * 255).to_num();
    let v: u16 = (v * 255).to_num();
    let f: u16 = (h * 2 % 85) * 3; // relative interval

    let p: u16 = v * (255 - s) / 255;
    let q: u16 = v * (255 - (s * f) / 255) / 255;
    let t: u16 = v * (255 - (s * (255 - f)) / 255) / 255;
    match h as u8 {
        0..=42 => RGB8 {
            r: v as u8,
            g: t as u8,
            b: p as u8,
        },
        43..=84 => RGB8 {
            r: q as u8,
            g: v as u8,
            b: p as u8,
        },
        85..=127 => RGB8 {
            r: p as u8,
            g: v as u8,
            b: t as u8,
        },
        128..=169 => RGB8 {
            r: p as u8,
            g: q as u8,
            b: v as u8,
        },
        170..=212 => RGB8 {
            r: t as u8,
            g: p as u8,
            b: v as u8,
        },
        213..=254 => RGB8 {
            r: v as u8,
            g: p as u8,
            b: q as u8,
        },
        255 => RGB8 {
            r: v as u8,
            g: t as u8,
            b: p as u8,
        },
    }
}
