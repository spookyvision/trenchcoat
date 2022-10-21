use std::time::Instant;

use super::traits::{Peripherals, PixelBlazeRuntime, TimerMs};
use crate::forth::bytecode::CellData;

#[derive(Debug, Clone)]
pub struct StdTimer {
    start: Instant,
}

impl Default for StdTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl StdTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl TimerMs for StdTimer {
    fn time_millis(&self) -> u32 {
        Instant::now().duration_since(self.start).as_millis() as u32
    }
}

#[derive(Debug, Clone, Default)]

pub(crate) struct Runtime {
    timer: StdTimer,
}

impl TimerMs for Runtime {
    fn time_millis(&self) -> u32 {
        self.timer.time_millis()
    }
}

impl Peripherals for Runtime {
    fn led_begin(&mut self) {
        println!("LED begin");
    }

    fn led_commit(&mut self) {
        println!("LED commit");
    }

    fn led_hsv(&mut self, idx: CellData, h: CellData, s: CellData, v: CellData) {
        println!("LED[{idx}] HSV({h},{s},{v})");
    }
}
