extern crate serialport;

mod serial;
use serial::communication::comms;

fn main() {
    // Did the user pass in a COM port?
    let user_requested_port = std::env::args().nth(1);

    // Try to get a handle on the port. Fail loudly.
    if let Some(port) = comms::get_serial_port(user_requested_port) {
        println!("Got a port named {:?}", port.name());
    } else {
        println!("Could not find a serial port with the appropriate device.");
        std::process::exit(1);
    }
}
