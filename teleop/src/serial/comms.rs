pub mod comms {
    use commands;
    use serialport;
    use std::sync::mpsc;

    /// Communicate with the device by listening on a channel from the console
    /// thread and sending the received commands over UART.
    /// Closes its resources and quits running when it receives the special
    /// quit command.
    pub fn communicate_with_device(port: Box<serialport::SerialPort>, rx: mpsc::Receiver<commands::Command>) {
        let mut should_quit = false;
        while !should_quit {
            // TODO:
            // Get the next command
            // Send it over UART as a string
        }
    }
}