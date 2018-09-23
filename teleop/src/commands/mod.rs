pub enum ServoId {
    ServoBase,
    ServoShoulder,
    ServoElbow,
    ServoWrist,
    ServoHand,
}

pub enum Command {
    Quit,
    Led(bool),              //on/off
    Servo(ServoId, u16),    // ServoID, angle
}