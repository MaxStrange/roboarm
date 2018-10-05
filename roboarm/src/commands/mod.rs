use super::servos::ServoId;

pub enum Command {
    Help,
    Led(bool),              // on/off
    Servo(ServoId, u16),    // ServoID, angle
}