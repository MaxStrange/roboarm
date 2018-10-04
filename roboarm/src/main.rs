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
use tm4c123x_hal as tm;
use tm4c123x_hal::prelude::*;
use tm4c123x_hal::serial::{NewlineMode, Serial};
use tm4c123x_hal::sysctl;

mod console;


#[entry]
fn main() -> ! {
    let mut periph = tm4c123x_hal::Peripherals::take().unwrap();
    // Now divvy out periph

    let mut sc = periph.SYSCTL.constrain();
    // Now divvy out sc

    sc.clock_setup.oscillator = sysctl::Oscillator::Main(
        sysctl::CrystalFrequency::_16mhz,
        sysctl::SystemClock::UsePll(sysctl::PllOutputFrequency::_80_00mhz),
    );
    let clocks = sc.clock_setup.freeze();

    // Get the uart stuff
    let uart0 = periph.UART0;
    let mut porta = periph.GPIO_PORTA.split(&sc.power_control);
    let uart0_tx = porta.pa1.into_af_push_pull::<tm4c123x_hal::gpio::AF1>(&mut porta.control);
    let uart0_rx = porta.pa0.into_af_push_pull::<tm4c123x_hal::gpio::AF1>(&mut porta.control);
    let mut uart = Serial::uart0(uart0, uart0_tx, uart0_rx, (), (), 115200_u32.bps(), NewlineMode::SwapLFtoCRLF, &clocks, &sc.power_control);

    let mut coreperiph = tm4c123x_hal::CorePeripherals::take().unwrap();
    // Now divvy out coreperiph


    // Set up the interrupts
    let mut nvic = coreperiph.NVIC;
    // TODO

    // TODO: enable Ssi?

    let mut con = console::Console::new(uart).unwrap();

    loop {
        con.serial.write_all("Hello, World!".as_bytes());
    }
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
