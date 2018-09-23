pub mod user_input {
    use commands;
    use std::io;
    use std::sync::mpsc;

    /// Reads lines from the user until the quit command is given.
    /// Attempts to parse the line into a valid command. If it fails,
    /// will pipe something useful to the user over stdout. If succeeds,
    /// gives the resultant command to the serial channel.
    pub fn read_from_user_until_quit(tx: mpsc::Sender<commands::Command>) {
        let mut should_quit = false;
        while !should_quit {
            let mut input = String::new();
            let parsed = match io::stdin().read_line(&mut input) {
                Ok(_nbytes) => {
                    commands::Command::new_from_string(&input)
                },
                Err(er) => {
                    println!("Error!: {}", er);
                    Err("")
                },
            };

            if let Ok(cmd) = parsed {
                match cmd {
                    commands::Command::Quit => {
                        should_quit = true;
                        tx.send(cmd).expect("Couldn't send the message to the Serial thread.");
                    },
                    commands::Command::Script(fpath) => {
                        run_script(&tx, &fpath);
                    },
                    _ => tx.send(cmd).expect("Couldn't send the message to the Serial thread."),
                }
            } else {
                println!("Error parsing input.");
            }
        }
    }

    /// Opens the given file, reads its contents, then executes each line as if it were
    /// a command entered into the console. Does not accept Quit commands or other script commands.
    fn run_script(tx: &mpsc::Sender<commands::Command>, fpath: &str) {
        // TODO: execute each line in the script
    }
}
