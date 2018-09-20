#![no_std]
#![no_main]

//extern crate cortex_m;
#[macro_use]                // Brings in entry! macro from Cortex Runtime
extern crate cortex_m_rt;   // Brings in the Cortex Runtime, including panic stuff
//#[macro_use]
extern crate tm4c123x_hal;  // Brings in SVD2Rust stuff, including ISR default implementations and peripheral structs

use core::panic::PanicInfo;
use tm4c123x_hal::prelude::*;
use tm4c123x_hal::serial::{NewlineMode, Serial};

mod console;

entry!(main);
fn main() -> ! {
    let periph = tm4c123x_hal::Peripherals::take().unwrap();
    let coreperiph = tm4c123x_hal::CorePeripherals::take().unwrap();
    loop {}
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
