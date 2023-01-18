use serde::Deserialize;


#[derive(Deserialize)]
pub struct GameLog {
    pub items: Vec<LogEvent>,
    pub next_page: String
}

#[derive(Deserialize)]
pub struct LogEvent {
    pub game_id: String,
    pub timestamp: String,
    pub data: GameEventData
}

#[derive(Deserialize)]
pub struct GameEventData {
    pub changedState: serde_json::Value,
    pub displayDelay: u32,
    pub displayOrder: u32,
    pub displayText: String,
    pub displayTime: String
}