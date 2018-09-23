/// Module mostly useful for providing convient functions for getting a new SerialPort object.
pub mod portcomms {
    use serialport;
    use testport;
    use std::time::Duration;

    const FTDI_2232H_VID: u16 = 0x0403;
    const FTDI_2232H_PID: u16 = 0x6010;

    /// Builds a new instance of SerialPortSettings, using the default settings for this program.
    /// Returns the struct by ownership.
    fn build_default_port_settings() -> serialport::SerialPortSettings {
        serialport::SerialPortSettings {
            baud_rate:      115200,
            data_bits:      serialport::DataBits::Eight,
            flow_control:   serialport::FlowControl::None,
            parity:         serialport::Parity::None,
            stop_bits:      serialport::StopBits::One,
            timeout:        Duration::from_millis(30),
        }
    }

    /// Opens a serial port by using preset settings and the given serial port info.
    fn open_serial_port(info: serialport::SerialPortInfo) -> Option<Box<serialport::SerialPort>> {
        let result = serialport::open_with_settings(&info.port_name, &build_default_port_settings());
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
    fn get_serial_port_by_vidpid(ports: Vec<serialport::SerialPortInfo>) -> Option<Box<serialport::SerialPort>> {
        // result should be the first item with the appropriate VID and PID
        for p in ports {
            let dup = p.clone();
            match dup.port_type {
                serialport::SerialPortType::UsbPort(info) => {
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
    /// If the user has requested a particular com port, that one is tried first.
    /// If the special string 'test' is passed in, we give a test port.
    pub fn get_serial_port(user_requested_port: Option<String>) -> Option<Box<serialport::SerialPort>> {
        // If the user has requested a port
        if let Some(comname) = user_requested_port {
            // Check if it is the test port
            if comname.trim().to_ascii_lowercase() == "test" {
                return Some(Box::new(testport::TestPort));
            }

            // If it is a real port, try opening it
            if let Ok(ret) = serialport::open_with_settings(comname.as_str(), &build_default_port_settings()) {
                return Some(ret);
            } else {
                println!("Could not get a serial port at device path {}, trying to find by VID/PID instead.", comname.as_str());
            }
        }

        // Try default look up mechanism as last resort
        if let Ok(ports) = serialport::available_ports() {
            match ports.len() {
                0 => None,
                _n => get_serial_port_by_vidpid(ports),
            }
        } else {
            panic!("Error listing serial ports.");
        }
    }
}
