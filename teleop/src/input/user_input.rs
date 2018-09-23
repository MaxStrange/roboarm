pub mod user_input {
    use commands;
    use std::sync::mpsc;

    /// Reads lines from the user until the quit command is given.
    /// Attempts to parse the line into a valid command. If it fails,
    /// will pipe something useful to the user over stdout. If succeeds,
    /// gives the resultant command to the serial channel.
    pub fn read_from_user_until_quit(rx: mpsc::Sender<commands::Command>) {
        // TODO
    }
}