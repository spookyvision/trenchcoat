use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

pub(crate) struct Peri {
    ws: Ws2812Esp32Rmt,
    leds: Vec<RGB8>,
}

impl Peri {
    pub(crate) fn new(data_pin: i32, pixel_count: usize) -> Self {
        let ws =
            Ws2812Esp32Rmt::new(0, data_pin as u32).expect("could not initialize LED peripheral");

        let mut leds = Vec::with_capacity(pixel_count);

        for i in 0..pixel_count {
            leds.push(RGB8::default());
        }

        Self { ws, leds }
    }

    pub(crate) fn set_rgb(&mut self, idx: usize, rgb: RGB8) {
        self.leds[idx] = rgb;
    }

    pub(crate) fn flush(&mut self) {
        self.ws.write(self.leds.iter().cloned()).unwrap();
    }
}
