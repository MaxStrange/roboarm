extern crate serialport;

mod commands;

mod input;
use input::user_input::user_input;

mod serial;
use serial::comms::comms;
use serial::port::portcomms;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

/// Spawns a thread that runs the serial port and a thread that reads
/// commands from the console. Joins the threads once the user enters
/// the quit command.
fn spin(port: Box<serialport::SerialPort>) {
    let (tx, rx): (Sender<commands::Command>, Receiver<commands::Command>) = mpsc::channel();
    let commthread = thread::spawn(move || comms::communicate_with_device(port, rx));
    let inputthread = thread::spawn(move || user_input::read_from_user_until_quit(tx));

    commthread.join();
    inputthread.join();
}

fn main() {
    // Did the user pass in a COM port?
    let user_requested_port = std::env::args().nth(1);

    // Try to get a handle on the port. Fail loudly.
    if let Some(port) = portcomms::get_serial_port(user_requested_port) {
        println!("Got a port named {:?}", port.name());
        spin(port);
    } else {
        println!("Could not find a serial port with the appropriate device.");
        std::process::exit(1);
    }
}
