pub mod comms {
    use commands;
    use serialport;
    use std::io;
    use std::sync::mpsc;

    /// Communicate with the device by listening on a channel from the console
    /// thread and sending the received commands over UART.
    /// Closes its resources and quits running when it receives the special
    /// quit command.
    pub fn communicate_with_device(mut port: Box<serialport::SerialPort>, rx: mpsc::Receiver<commands::Command>) {
        let mut should_quit = false;
        while !should_quit {
            // Get the next command
            let cmd = rx.recv().expect("Problem reading from the Input channel");

            match cmd {
                commands::Command::Help => panic!("Should not have gotten help command on this thread."),
                commands::Command::Quit => { should_quit = true; },
                commands::Command::Script(_) => panic!("Should not have gotten script command on this thread."),
                commands::Command::Led(_on) => { write_to_port(&mut port, cmd).unwrap(); },
                commands::Command::Servo(_id, _angle) => { write_to_port(&mut port, cmd).unwrap(); },
            }
        }
    }

    /// Attempts to write the command to the port. Returns whether or not it worked.
    fn write_to_port(port: &mut Box<serialport::SerialPort>, cmd: commands::Command) -> io::Result<usize> {
        match cmd {
            commands::Command::Led(on) => {
                let onoff = if on { 1 } else { 0 };
                let msg = format!("led {}", onoff);
                port.write(msg.as_bytes())
            },
            commands::Command::Servo(id, angle) => {
                let msg = format!("servo {} {}", id as u8, angle);
                port.write(msg.as_bytes())
            },
            commands::Command::Quit => panic!("Should not have gotten quit command in 'write_to_port'"),
            commands::Command::Script(_) => panic!("Should not have gotten script command in 'write_to_port'"),
            commands::Command::Help => panic!("Should not have gotten help command in 'write_to_port'"),
        }
    }
}
