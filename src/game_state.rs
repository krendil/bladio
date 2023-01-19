use std::sync::mpsc::{Sender, Receiver};
use std::thread::{JoinHandle, self};

use crate::announce_channel::AnnounceEvent;
use crate::events::{Team, GameEvent, PlayEvent, Inning};

pub struct GameState {

    announce: Sender<AnnounceEvent>,

    home_team: Team,
    away_team: Team,

    home_score: i32,
    away_score: i32
}

impl GameState {

    fn new(announce: Sender<AnnounceEvent>) -> GameState {
        return GameState {
            announce: announce,
            home_team: Team {
                full_name: "Default Team".to_string(),
                short_name: "Default".to_string(),
            },
            away_team: Team {
                full_name: "Default Team".to_string(),
                short_name: "Default".to_string()
            },

            home_score: 0,
            away_score: 0
        }
    }

    fn pregame(mut self, home: Team, away: Team) -> GameState {
        self.home_team = home;
        self.away_team = away;
        // Announce upcoming game
        let message = format!("Tuning in to: {} vs {}.", self.home_team.full_name, self.away_team.full_name);
        self.announce.send(AnnounceEvent::Message(message)).unwrap();
        return self;
    }

    fn play_ball(self) -> GameState  {
        let message = format!("Play ball!");
        self.announce.send(AnnounceEvent::Message(message)).unwrap();
        return self;
    }

    fn play_event(mut self, play_event: PlayEvent) -> GameState  {
        match play_event.home_score {
            Some(score) => self.home_score = score,
            None => ()
        }

        match play_event.away_score {
            Some(score) => self.away_score = score,
            None => ()
        }

        if play_event.thwack > 0.0 {
            self.announce.send(AnnounceEvent::Thwack(play_event.thwack)).unwrap();
            self.announce.send(AnnounceEvent::Beat()).unwrap();
        }

        self.announce.send(AnnounceEvent::Message(play_event.message)).unwrap();
        self.announce.send(AnnounceEvent::Beat()).unwrap();
        return self;
    }

    fn inning_end(self, inning: Inning) -> GameState  {
        let message = format!("End of the {0:?} of the {1}. {2} {3}, {4} {5}.",
            inning.end, inning.number,
            self.home_team.short_name, self.home_score,
            self.away_team.short_name, self.away_score);
        self.announce.send(AnnounceEvent::Message(message)).unwrap();
        self.announce.send(AnnounceEvent::Beat()).unwrap();
        return self;
    }

    fn game_end(self) -> GameState  {
        let message = format!("Game over. {} {}, {} {}.",
            self.home_team.full_name, self.home_score,
            self.away_team.full_name, self.away_score);
        self.announce.send(AnnounceEvent::Message(message)).unwrap();

        return self;
    }

    fn end_broadcast(self) -> GameState {
        self.announce.send(AnnounceEvent::Finish()).unwrap();
        return self;
    }


}

pub fn spawn_game_thread(rx: Receiver<GameEvent>, tx: Sender<AnnounceEvent>) -> JoinHandle<i32> {
    return thread::spawn(move || {
        game_loop(rx, tx);
        return 0;
    });
}

fn game_loop(rx: Receiver<GameEvent>, tx: Sender<AnnounceEvent>) {
    let mut game = GameState::new(tx);
    loop {
        let ev = rx.recv().unwrap_or(GameEvent::EndBroadcast());
        game = match ev {
            GameEvent::Pregame(home, away) => game.pregame(home, away),
            GameEvent::PlayBall() => game.play_ball(),
            GameEvent::PlayEvent(play_event) => game.play_event(play_event),
            GameEvent::InningEnd(inning) => game.inning_end(inning),
            GameEvent::GameEnd() => game.game_end(),
            GameEvent::EndBroadcast() => { game.end_broadcast(); break; }
        }
    }
}