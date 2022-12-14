#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::{
    alloc::Layout,
    sync::atomic::{AtomicUsize, Ordering},
};

use defmt_rtt as _; // global logger
// TODO adjust HAL import
// use some_hal as _; // memory layout
use panic_probe as _;

pub mod runtime;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", {
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n
});

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    panic!("the heap is too damn full");
}
