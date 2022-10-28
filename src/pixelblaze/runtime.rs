// TODO WasmRuntime hsv console.log blah

use super::traits::Peripherals;
use crate::forth::{runtime::CoreRuntime, vm::CellData};

pub struct ConsoleRuntime {
    time_ms: u32,
    dt: u32,
    led_idx: usize,
}

impl ConsoleRuntime {
    pub fn new(dt: u32) -> Self {
        Self {
            time_ms: 0,
            dt,
            led_idx: 0,
        }
    }
}

impl Default for ConsoleRuntime {
    fn default() -> Self {
        Self::new(100)
    }
}

impl Peripherals for ConsoleRuntime {
    fn led_begin(&mut self) {
        println!("LED begin");
    }

    fn led_commit(&mut self) {
        println!("LED commit");
        println!("inc time by {}ms", self.dt);
        self.time_ms += self.dt;
    }

    fn led_hsv(&mut self, h: CellData, s: CellData, v: CellData) {
        println!("LED[{}] HSV({h},{s},{v})", self.led_idx);
    }

    fn set_led_idx(&mut self, idx: usize) {
        self.led_idx = idx;
    }
}

impl CoreRuntime for ConsoleRuntime {
    fn time_millis(&mut self) -> u32 {
        self.time_ms
    }

    fn log(&mut self, s: &str) {
        println!("[LOG] {s}");
    }
}
