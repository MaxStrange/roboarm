use std::path;

pub enum ServoId {
    Base,
    Shoulder,
    Elbow,
    Wrist,
    Hand,
}

impl ServoId {
    pub fn from_i32(x: i32) -> Option<ServoId> {
        match x {
            x if x == ServoId::Base as i32 => Some(ServoId::Base),
            x if x == ServoId::Shoulder as i32 => Some(ServoId::Shoulder),
            x if x == ServoId::Elbow as i32 => Some(ServoId::Elbow),
            x if x == ServoId::Wrist as i32 => Some(ServoId::Wrist),
            x if x == ServoId::Hand as i32 => Some(ServoId::Hand),
            _ => None,
        }
    }
}

pub enum Command {
    Quit,
    Led(bool),              //on/off
    Servo(ServoId, u16),    // ServoID, angle
    Script(String),         // fpath
}

impl Command {
    /// Returns a new Command from a string. If the string cannot be parsed
    /// into one of the Command variants, returns an error message instead.
    pub fn new_from_string(line: &str) -> Result<Command, &'static str> {
        // If the string is empty or only whitespace, return
        if line.trim().is_empty() {
            return Err("Line is empty");
        }

        // Otherwise, try to match on the first item and route the parsing appropriately
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        assert!(tokens.len() > 0);
        match tokens[0].to_ascii_lowercase().as_str() {
            "quit" => Ok(Command::Quit),
            "led" => Command::led_from_string(line),
            "servo" => Command::servo_from_string(line),
            "script" => Command::script_from_string(line),
            _ => Err("Malformed command"),
        }
    }

    /// Returns a new Led command from the given string if possible. Otherwise
    /// returns an error message.
    pub fn led_from_string(line: &str) -> Result<Command, &'static str> {
        // If the string is empty, it is an error
        if line.trim().is_empty() {
            return Err("Line is empty");
        }

        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        assert!(tokens.len() > 0);

        // If the string's first token.lower() is not 'led', that's an error
        if tokens[0].to_ascii_lowercase() != "led" {
            return Err("Line does not start with LED");
        }

        // If the string does start with 'led', but it does not have a second token
        // that's also an error (or if it has more than two tokens)
        if tokens.len() != 2 {
            return Err("USAGE for LED command: LED <on/off>");
        }

        // If the string is two tokens, but the last one is not 'on' or 'off', that's an error
        assert!(tokens.len() == 2);
        match tokens[1].to_ascii_lowercase().as_str() {
            "on" => Ok(Command::Led(true)),
            "off" => Ok(Command::Led(false)),
             _ => Err("USAGE for LED command: LED <on/off>"),
        }
    }

    pub fn servo_from_string(line: &str) -> Result<Command, &'static str> {
        // If the string is empty, it is an error
        if line.trim().is_empty() {
            return Err("Line is empty");
        }

        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        assert!(tokens.len() > 0);

        // If the string's first token is not 'servo', that's an error
        if tokens[0].to_ascii_lowercase() != "servo" {
            return Err("Line does not start with servo");
        }

        // If the string does start with 'servo', but it does not have the right number of tokens,
        // that's also an error
        if tokens.len() != 3 {
            return Err("USAGE for Servo command: servo <id> <angle>")
        }

        // If the second token is not a valid servo ID, that's an error
        assert!(tokens.len() == 3);
        let servoid;
        if let Ok(id) = tokens[1].parse::<i32>() {
            let servoid_as_num = id;
            if let Some(val) = ServoId::from_i32(servoid_as_num as i32) {
                servoid = val;
            } else {
                return Err("Servo id is invalid.");
            }
        } else {
            return Err("USAGE for Servo command: servo <id - must be numeric> <angle - between 0 and 360>");
        }

        assert!(tokens.len() == 3);
        // If the third token is not a valid angle (0 to 360), that's also an error
        let angle = match tokens[2].parse::<f64>() {
            Ok(val) if val <= 360.0 && val >= 0.0 => val,
            Err(_) => { return Err("Need a numeric value for angle"); },
            Ok(_) => { return Err("Angle must be in range [0, 360]"); },
        };

        Ok(Command::Servo(servoid, angle.round() as u16))
    }

    /// Attempt to parse the line into 'script <fpath>'.
    pub fn script_from_string(line: &str) -> Result<Command, &'static str> {
        // If the string is empty, it is an error
        if line.trim().is_empty() {
            return Err("Line is empty");
        }

        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        assert!(tokens.len() > 0);

        // If the string's first token is not 'script', that's an error
        if tokens[0].to_ascii_lowercase() != "script" {
            return Err("Line does not start with script");
        }

        // If there is not exactly two tokens, that's an error
        if tokens.len() != 2 {
            return Err("USAGE: script <path-to-script>");
        }

        assert!(tokens.len() == 2);
        let fpath = path::Path::new(tokens[1]);

        // If the second token is not a valid file path, that's an error
        if !fpath.exists() {
            return Err("Given path does not point to a file");
        }

        Ok(Command::Script(fpath.to_str().unwrap().to_string()))
    }
}