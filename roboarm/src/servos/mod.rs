pub enum ServoId {
    Base,
    Shoulder,
    Elbow,
    Wrist,
    Hand,
}

impl ServoId {
    pub fn from_isize(x: isize) -> Option<ServoId> {
        match x {
            x if x == ServoId::Base as isize => Some(ServoId::Base),
            x if x == ServoId::Shoulder as isize => Some(ServoId::Shoulder),
            x if x == ServoId::Elbow as isize => Some(ServoId::Elbow),
            x if x == ServoId::Wrist as isize => Some(ServoId::Wrist),
            x if x == ServoId::Hand as isize => Some(ServoId::Hand),
            _ => None,
        }
    }
}
