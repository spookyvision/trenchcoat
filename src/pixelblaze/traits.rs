use crate::{forth::vm::CellData, vanillajs::runtime::VanillaJSRuntime};

pub trait Peripherals {
    fn led_begin(&mut self) {}

    // TODO this feels bleh, maybe better pass index in the executor (trait?)
    // and/or rethink the exact executor/runtime split, the contract is to iterate over all LEDs anyway
    //
    // in related news, shared memory should be a thing
    // (here we could e.g. do shm("surface")[0] = [r,g,b]);
    fn set_led_idx(&mut self, idx: usize);

    fn led_hsv(&mut self, h: CellData, s: CellData, v: CellData);
    fn led_rgb(&mut self, r: CellData, g: CellData, b: CellData);

    // TODO we're extending pixelblaze here.
    // strictly speaking this should be in a different namespace…
    // but how to compose those without drowning in generics?
    fn ext_led_okhsl(&mut self, h: CellData, s: CellData, l: CellData);

    fn led_commit(&mut self) {}
}

pub trait PixelBlazeRuntime: VanillaJSRuntime + Peripherals {}

impl<RT> PixelBlazeRuntime for RT where RT: VanillaJSRuntime + Peripherals {}
