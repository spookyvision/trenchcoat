use std::path::Path;

use trenchcoat::{
    forth::{
        compiler::{compile, Flavor, Source},
        vm::VM,
    },
    pixelblaze::{executor::Executor, ffi::PixelBlazeFFI, runtime::ConsoleRuntime},
    prelude::*,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let file = "../res/rainbow melt.js";

    let mut ser = compile(
        Source::File(Path::new(file).to_path_buf().into_boxed_path()),
        Flavor::Pixelblaze,
    )?;
    let vm: VM<PixelBlazeFFI, ConsoleRuntime> = postcard::from_bytes_cobs(&mut ser)?;

    let pixel_count = 1000;
    let mut executor = Executor::new(vm, pixel_count);
    executor.start()?;
    for _ in 0..1000 {
        executor.do_frame()?;
    }

    Ok(())
}
