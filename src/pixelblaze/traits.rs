use crate::forth::{runtime::CoreRuntime, vm::CellData};

pub trait Peripherals {
    fn led_begin(&mut self) {}
    fn set_led_idx(&mut self, idx: usize);
    fn led_hsv(&mut self, h: CellData, s: CellData, v: CellData);
    fn led_commit(&mut self) {}
}

pub trait PixelBlazeRuntime: CoreRuntime + Peripherals {}

impl<RT> PixelBlazeRuntime for RT where RT: CoreRuntime + Peripherals {}
