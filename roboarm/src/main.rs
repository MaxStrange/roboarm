#![no_std]
#![no_main]

/**
 * Much initialization code stolen most gleefully from jpster: https://github.com/thejpster/monotron
 * Go check out that project, since it's awesome.
 */

//extern crate cortex_m;
#[macro_use]                // Brings in entry! macro from Cortex Runtime
extern crate cortex_m_rt;   // Brings in the Cortex Runtime, including panic stuff
//#[macro_use]
extern crate tm4c123x_hal;  // Brings in SVD2Rust stuff, including ISR default implementations and peripheral structs

use core::panic::PanicInfo;
use tm4c123x_hal::prelude::*;
use tm4c123x_hal::serial::{NewlineMode, Serial};
use tm4c123x_hal::sysctl;

mod console;

#[entry]
fn main() -> ! {
    let periph = tm4c123x_hal::Peripherals::take().unwrap();
    let coreperiph = tm4c123x_hal::CorePeripherals::take().unwrap();
    let mut sc = periph.SYSCTL.constrain();

    // Set up the system clock
    sc.clock_setup.oscillator = sysctl::Oscillator::Main(
        sysctl::CrystalFrequency::_16mhz,
        sysctl::SystemClock::UsePll(sysctl::PllOutputFrequency::_80_00mhz),
    );
    let clocks = sc.clock_setup.freeze();

    // Set up the interrupts
    let mut nvic = coreperiph.NVIC;
    // TODO

    // TODO: enable Ssi?

    // TODO: enable GPIO ports as needed
    let mut porta = periph.GPIO_PORTA.split(&sc.power_control);

    // UART
    let mut uart = Serial::uart0(
        periph.UART0,
        porta.pa1.into_af_push_pull::<tm4c123x_hal::gpio::AF1>(&mut porta.control), //tx
        porta.pa0.into_af_push_pull::<tm4c123x_hal::gpio::AF1>(&mut porta.control), //rx
        (), //rts
        (), //cts
        115200_u32.bps(),
        NewlineMode::SwapLFtoCRLF,
        &clocks,
        &sc.power_control,
    );

    loop {
        uart.write_all("Hello, World!".as_bytes());
    }
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
