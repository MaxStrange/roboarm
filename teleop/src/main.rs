extern crate serialport;

mod commands;

mod input;
use self::input::user_input::user_input;

mod serial;
use self::serial::comms::comms;
use self::serial::port::portcomms;
use self::serial::testport;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

fn main() {
    // Did the user pass in a COM port?
    let user_requested_port = std::env::args().nth(1);

    // Try to get a handle on the port. Fail loudly.
    if let Some(port) = portcomms::get_serial_port(user_requested_port) {
        println!("Got a port named {:?}", port.name());

        // Did the user pass in a script?
        let script = std::env::args().nth(2);

        if let Some(script) = script {
            println!("Executing script {:?}", script);
            run_script(port, script);
        } else {
            println!("Executing spin");
            spin(port);
        }
    } else {
        println!("Could not find a serial port with the appropriate device.");
        std::process::exit(1);
    }
}

/// Spawns a thread that runs the serial port and a thread that reads
/// commands from the console. Joins the threads once the user enters
/// the quit command.
fn spin(port: Box<serialport::SerialPort>) {
    let (tx, rx): (Sender<commands::Command>, Receiver<commands::Command>) = mpsc::channel();
    let commthread = thread::spawn(move || comms::communicate_with_device(port, rx));
    let inputthread = thread::spawn(move || user_input::read_from_user_until_quit(tx));

    if let Err(msg) = commthread.join() {
        println!("Problem joining comm thread: {:?}", msg);
    }
    if let Err(msg) = inputthread.join() {
        println!("Problem joining input thread: {:?}", msg);
    }
}

fn run_script(port: Box<serialport::SerialPort>, scriptpath: String) {
    let (tx, rx): (Sender<commands::Command>, Receiver<commands::Command>) = mpsc::channel();
    let commthread = thread::spawn(move || comms::communicate_with_device(port, rx));

    if let Err(msg) = user_input::run_script(&tx, scriptpath.as_str()) {
        println!("Problem running script:\n{}", msg);
        std::process::exit(2);
    }

    if let Err(msg) = commthread.join() {
        println!("Problem joining comm thread: {:?}", msg);
    }
}
