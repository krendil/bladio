use std::sync::mpsc::{Sender, Receiver};
use std::thread::{JoinHandle, spawn};
use std::path::Path;
use std::fs::File;

use serde_json::{StreamDeserializer};

use crate::events::PlayEvent;

mod json_types;

pub fn new(filename: &str, tx: Sender<PlayEvent>) -> JoinHandle<i32> {
    let filepath = Path::new(filename);
    let file = File::open(filepath).unwrap();
    return spawn(move || {
        load_events_from_file(&file, tx);
        return 0;
    });
}

fn load_events_from_file(file: &File, tx: Sender<PlayEvent>) {
    let log: json_types::GameLog = serde_json::from_reader(file).unwrap();
    for item in log.items {

        let event: PlayEvent = PlayEvent {
            message: item.data.displayText,
            hit: false,
            yay: 0.0,
            oh: 0.0
        };

        tx.send(event).unwrap();
    }
}