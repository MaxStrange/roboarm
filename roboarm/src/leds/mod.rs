mod led;

use core::sync::atomic;
use self::led::led::Color;
use self::led::led::Led;
use tm4c123x_hal as tm;

type pf1 = tm::gpio::gpiof::PF1<tm::gpio::AlternateFunction<tm::gpio::AF1, tm::gpio::PushPull>>; //TODO: Should not be alternate function
type pf2 = tm::gpio::gpiof::PF2<tm::gpio::AlternateFunction<tm::gpio::AF1, tm::gpio::PushPull>>;
type pf3 = tm::gpio::gpiof::PF3<tm::gpio::AlternateFunction<tm::gpio::AF1, tm::gpio::PushPull>>;

/// Whether or not we have checked out the SystemLeds singleton
static CHECKED_OUT: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

pub struct SystemLeds {
    red: Led,
    green: Led,
    blue: Led,
}

impl SystemLeds {
    pub fn new(red: pf1, green: pf3, blue: pf2) -> Option<SystemLeds> {
        if CHECKED_OUT.swap(true, atomic::Ordering::Relaxed) {
            None
        } else {
            let r = Led{color: Color::Red};
            let g = Led{color: Color::Green};
            let b = Led{color: Color::Blue};
            Some(SystemLeds{red: r, green: g, blue: b})
        }
    }
}
