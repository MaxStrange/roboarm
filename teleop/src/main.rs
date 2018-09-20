extern crate serialport;
use serialport::prelude::*;
use serialport::SerialPortType;
use std::time::Duration;

const FTDI_2232H_VID: u16 = 0x0403;
const FTDI_2232H_PID: u16 = 0x6010;

/// Opens a serial port by using preset settings and the given serial port info.
fn open_serial_port(info: SerialPortInfo) -> Option<Box<SerialPort>> {
    let s = SerialPortSettings {
        baud_rate:      9600,
        data_bits:      DataBits::Eight,
        flow_control:   FlowControl::None,
        parity:         Parity::None,
        stop_bits:      StopBits::One,
        timeout:        Duration::from_millis(30),
    };

    let result = serialport::open_with_settings(&info.port_name, &s);
    match result {
        Ok(port) => Some(port),
        Err(e)   => {
            println!("Error when opening the port {}: {:?}", info.port_name, e);
            None
        },
    }
}

/// Returns Some(open serial port) or None, by finding and opening the appropriate port
/// from a list of serial port info objects.
fn get_serial_port_by_vidpid(ports: Vec<SerialPortInfo>) -> Option<Box<SerialPort>> {
    // result should be the first item with the appropriate VID and PID
    for p in ports {
        let dup = p.clone();
        match dup.port_type {
            SerialPortType::UsbPort(info) => {
                if info.vid == FTDI_2232H_VID && info.pid == FTDI_2232H_PID {
                    let result = open_serial_port(p);
                    return result;
                }
            }
            _ => (),
        }
    }
    None
}

/// Get the serial port to the robot arm or None.
fn get_serial_port() -> Option<Box<SerialPort>> {
    if let Ok(ports) = serialport::available_ports() {
        let result = match ports.len() {
            0 => None,
            _n => get_serial_port_by_vidpid(ports),
        };
        return result;
    } else {
        panic!("Error listing serial ports.");
    }
}

fn main() {
    if let Some(port) = get_serial_port() {
        println!("Got a port named {:?}", port.name());
    } else {
        println!("Could not find a serial port with the appropriate device.");
        std::process::exit(1);
    }
}
