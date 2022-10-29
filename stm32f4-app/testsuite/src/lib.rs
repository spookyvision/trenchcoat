#![no_std]
#![cfg_attr(test, no_main)]

use stm32f4_app as _; // memory layout + panic handler

#[defmt_test::tests]
mod tests {}
