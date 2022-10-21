use crate::forth::bytecode::CellData;

pub trait TimerMs {
    fn time_millis(&self) -> u32;
}

pub trait Peripherals {
    fn led_begin(&mut self) {}
    fn led_hsv(&mut self, idx: CellData, h: CellData, s: CellData, v: CellData);
    fn led_commit(&mut self) {}
}

pub trait PixelBlazeRuntime: TimerMs + Peripherals {}

impl<RT> PixelBlazeRuntime for RT where RT: Peripherals + TimerMs {}
