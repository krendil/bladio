use std::ops::Not;

#[derive(Debug)]
pub struct Team {
    pub full_name: String,
    pub short_name: String
}

#[derive(Debug)]
pub enum Side {
    Home,
    Away
}

#[derive(Debug)]
pub struct Inning {
    pub wasTop: bool,
    pub number: i32
}

#[derive(Debug)]
pub struct PlayEvent {
    // Message for the announcer to read
    pub message: String,
    // Whether and how loud to play a baseball hit sound before the message
    pub thwack: f32,
    // How loud happy crowd cheering should be, 0-1
    pub yay: f32,
    // How loud disappointed crowd cheering should be, 0-1
    pub oh: f32,
    // Optional: Update home/away scores
    pub home_score: Option<i32>,
    pub away_score: Option<i32>
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

