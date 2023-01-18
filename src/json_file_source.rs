use std::sync::mpsc::{Sender, Receiver};
use std::thread::{JoinHandle, spawn};
use std::path::Path;
use std::fs::File;
use regex::Regex;
use lazy_static::lazy_static;

use serde::de::value::BoolDeserializer;
use serde_json::{StreamDeserializer};

use crate::events::{PlayEvent, GameEvent, Team, End, Inning};
use crate::json_file_source::json_types::GameEventData;

mod json_types;

pub fn new(filename: &str, tx: Sender<GameEvent>) -> JoinHandle<i32> {
    let filepath = Path::new(filename);
    let file = File::open(filepath).unwrap();
    return spawn(move || {
        load_events_from_file(&file, tx);
        return 0;
    });
}

fn load_events_from_file(file: &File, tx: Sender<GameEvent>) {
    let log: json_types::GameLog = serde_json::from_reader(file).unwrap();

    // These dumps don't contain match info, so hardcode it
    tx.send(GameEvent::Pregame(Team {
        full_name: "Breckenridge Jazz Hands".to_string(), short_name: "Jazz Hands".to_string()
    }, Team{
        full_name: "Dallas Steaks".to_string(), short_name: "Steaks".to_string()
    })).unwrap();

    for item in log.items {

        match translate_event(item.data) {
            Some(event) => tx.send(event).unwrap(),
            None => ()
        }

    }

    tx.send(GameEvent::EndBroadcast()).unwrap();
}

fn extract_i32(data: &GameEventData, label: &str) -> Option<i32> {
    return match data.changedState.get(label) {
        Some(serde_json::Value::Number(n)) => Some(n),
        _ => None
    }.and_then(|n| {
        return n.as_i64();
    }).and_then(|n| {
        return Some(n as i32);
    });
}

fn is_complete(data: &GameEventData) -> bool {
    return data.changedState.get("complete").map_or(false, |b| {
        match b {
            serde_json::Value::Bool(b) => b.to_owned(),
            _ => false
        }
    });
}

fn get_thwack(data: &GameEventData) -> f32 {
    lazy_static! {
        static ref REGEX: Regex = Regex::new(
        "(A [^ ]* hit to(ward)? .*\\.\\.\\.)|(.* ((hits)|(swats)|(slaps)|(rolls)|(drags)|(chops)|(thumps)|(bats)|(knocks)|(sputters)|(taps)|(pushes)) ((it)|(the pitch)|(the ball)|(one)) (in)?to(wards?)? .*\\.\\.\\.)"
        ).unwrap();
    }

    // Detect fouls
    if data.displayText.starts_with("Foul ball") 
    || data.displayText.contains(" fouls it ")
    || data.displayText.contains(" hits a foul"){
        return 0.5;
    } else if REGEX.is_match(&data.displayText) {
        return 1.0;
    }
    return 0.0;
}

fn translate_event(data: GameEventData) -> Option<GameEvent> {

    if is_complete(&data) {
        return Some(GameEvent::GameEnd());
    } else if data.changedState.contains_key("topOfInning") {
        return extract_i32(&data, "inning")
            .zip(
                match data.changedState["topOfInning"] {
                    serde_json::Value::Bool(b @ true) => Some(End::Top),
                    serde_json::Value::Bool(b @ false) => Some(End::Bottom),
                    _ => None
                }
            ).and_then(|(inning, end)| {
                return Some(GameEvent::InningEnd(Inning{
                    number: inning,
                    end: end,
                }));
            })
        ;
    }
    else if data.displayText.is_empty() {
        return None;
    }
    else if data.displayText.eq("Play Ball!") {
        return Some(GameEvent::PlayBall());
    } else {
        let home_score = extract_i32(&data, "homeScore");
        let away_score = extract_i32(&data, "awayScore");
        let thwack = get_thwack(&data);
        let event: PlayEvent = PlayEvent {
            message: data.displayText,
            thwack: thwack,
            yay: 0.0,
            oh: 0.0,
            home_score: home_score,
            away_score: away_score
        };
        return Some(GameEvent::PlayEvent(event));
    }
}