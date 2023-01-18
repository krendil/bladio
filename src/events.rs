pub struct PlayEvent {
    // Message for the announcer to read
    pub message: String,
    // Whether to play a baseball hit sound before the message
    pub hit: bool,
    // How loud happy crowd cheering should be, 0-1
    pub yay: f32,
    // How loud disappointed crowd cheering should be, 0-1
    pub oh: f32,
}