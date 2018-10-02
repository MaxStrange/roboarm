use core::sync::atomic;
use tm4c123x_hal as tm;
use tm4c123x_hal::prelude::*;
use tm4c123x_hal::serial::{NewlineMode, Serial};

type TxPin = tm::gpio::gpioa::PA1<tm::gpio::AlternateFunction<tm::gpio::AF1, tm::gpio::PushPull>>;
type RxPin = tm::gpio::gpioa::PA0<tm::gpio::AlternateFunction<tm::gpio::AF1, tm::gpio::PushPull>>;

/// Whether or not we have checked out the Console singleton
static CHECKED_OUT: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

/// Console takes care of all things related to the console.
/// The typical usage is for the main module to initialize a Console struct (a singleton),
/// by using the appropriate builder pattern, then to invoke the console's run() function
/// each tick of the main loop.
pub struct Console {
    pub serial: Serial<tm::serial::UART0, TxPin, RxPin, (), ()>,
}

impl Console {
    pub fn new(s: Serial<tm::serial::UART0, TxPin, RxPin, (), ()>) -> Option<Console> {
        if CHECKED_OUT.swap(true, atomic::Ordering::Relaxed) {
            None
        } else {
            Some(Console{serial: s})
        }
    }
}