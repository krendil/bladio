#[derive(Debug)]
pub struct Team {
    pub full_name: String,
    pub short_name: String
}

#[derive(Debug)]
pub enum End {
    Top,
    Bottom
}

#[derive(Debug)]
pub struct Inning {
    pub end: End,
    pub number: i32
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum GameEvent {
    Pregame(Team, Team),
    PlayBall(),
    PlayEvent(PlayEvent),
    InningEnd(Inning),
    GameEnd(),
    EndBroadcast()
}

