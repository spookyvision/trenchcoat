use super::traits::Peripherals;
use crate::{
    forth::{compiler::MockRuntime, vm::CellData},
    vanillajs::runtime::VanillaJSRuntime,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsoleRuntime {
    time_ms: u32,
    dt: i32,
    led_idx: usize,
}

impl ConsoleRuntime {
    pub fn new(dt: i32) -> Self {
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
        trench_debug!("LED begin");
    }

    fn led_commit(&mut self) {
        trench_debug!("LED commit");
        trench_debug!("step time by {}ms", self.dt);
        self.time_ms = self.time_ms.wrapping_add_signed(self.dt);
    }

    fn led_hsv(&mut self, h: CellData, s: CellData, v: CellData) {
        let h: f32 = h.to_num();
        let s: f32 = s.to_num();
        let v: f32 = v.to_num();
        trench_debug!("LED[{}] HSV({},{},{})", self.led_idx, h, s, v);
    }

    fn set_led_idx(&mut self, idx: usize) {
        self.led_idx = idx;
    }
}

impl VanillaJSRuntime for ConsoleRuntime {
    fn time_millis(&mut self) -> u32 {
        self.time_ms
    }

    fn log(&mut self, s: &str) {
        trench_debug!("[LOG] {}", s);
    }
}

impl Peripherals for MockRuntime {
    fn set_led_idx(&mut self, idx: usize) {
        unimplemented!()
    }

    fn led_hsv(&mut self, h: CellData, s: CellData, v: CellData) {
        unimplemented!()
    }
}
