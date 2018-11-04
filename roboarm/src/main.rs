#![no_std]
#![no_main]

/**
 * Much initialization code stolen most gleefully from jpster: https://github.com/thejpster/monotron
 * Go check out that project, since it's awesome.
 */

/* Externs */
extern crate panic_halt;

/* Use Statements */
use tm4c123x_hal as tm;

use cortex_m_rt::entry;
use self::tm::prelude::*;
use self::tm::serial::{NewlineMode, Serial};
use self::tm::sysctl;

/* Mod Declarations */
mod commands;
mod console;
mod leds;
mod servos;


fn init() -> (console::Console, tm::delay::Delay, leds::SystemLeds) {
    /* Take all the peripherals in the system */
    let periph = tm4c123x_hal::Peripherals::take().unwrap();

    /* Pull out the systemctl block */
    let mut sc = periph.SYSCTL.constrain();

    /* Set up the clocks */
    sc.clock_setup.oscillator = sysctl::Oscillator::Main(
        sysctl::CrystalFrequency::_16mhz,
        sysctl::SystemClock::UsePll(sysctl::PllOutputFrequency::_80_00mhz),
    );
    let clocks = sc.clock_setup.freeze();

    /* Initialize the UART */
    let uart0 = periph.UART0;
    let mut porta = periph.GPIO_PORTA.split(&sc.power_control);
    let uart0_tx = porta.pa1.into_af_push_pull::<tm::gpio::AF1>(&mut porta.control);
    let uart0_rx = porta.pa0.into_af_push_pull::<tm::gpio::AF1>(&mut porta.control);
    let uart = Serial::uart0(uart0, uart0_tx, uart0_rx, (), (), 115200_u32.bps(), NewlineMode::SwapLFtoCRLF, &clocks, &sc.power_control);

    /* Initialize the LEDs */
    let mut portf = periph.GPIO_PORTF.split(&sc.power_control);
    let red = portf.pf1.into_af_push_pull::<tm::gpio::AF1>(&mut portf.control);
    let green = portf.pf3.into_af_push_pull::<tm::gpio::AF1>(&mut portf.control);
    let blue = portf.pf2.into_af_push_pull::<tm::gpio::AF1>(&mut portf.control);

    /* Get the core peripherals and then start divvying them out */
    let mut coreperiph = tm::CorePeripherals::take().unwrap();

    /* Take the systick block */
    let systick = coreperiph.SYST;

    /* Set up all the interrupts */
    // TODO
    //let mut nvic = coreperiph.NVIC;

    /* Return all the initialized singletons */
    let con = console::Console::new(uart).unwrap();
    let delay = tm::delay::Delay::new(systick, &clocks);
    let sysleds = leds::SystemLeds::new(red, green, blue).unwrap();

    (con, delay, sysleds)
}

#[entry]
fn main() -> ! {
    let (mut con, delay, sysleds) = init();
    loop {
        con.run_statemachine();
    }
}
