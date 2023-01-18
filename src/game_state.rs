use std::sync::mpsc::{Sender, Receiver};
use std::thread::{JoinHandle, spawn, self};

use crate::events::{Team, GameEvent, PlayEvent, Inning};

pub struct GameState {

    home_team: Team,
    away_team: Team,

    home_score: i32,
    away_score: i32
}

impl GameState {

    fn new() -> GameState {
        return GameState {
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
        println!("Pregame: {} vs {}.", self.home_team.full_name, self.away_team.full_name);
        return self;
    }

    fn play_ball(self) -> GameState  {
        println!("Play ball!");
        return self;
    }

    fn play_event(self, play_event: PlayEvent) -> GameState  {
        println!("{}", play_event.message);
        return self;
    }

    fn inning_end(self, inning: Inning) -> GameState  {
        println!("End of the {0:?} of the {1}. {2} {3}, {4} {5}.",
            inning.end, inning.number,
            self.home_team.short_name, self.home_score,
            self.away_team.short_name, self.away_score);
        return self;
    }

    fn game_end(self) -> GameState  {
        println!("Game over. {} {}, {} {}.",
            self.home_team.short_name, self.home_score,
            self.away_team.short_name, self.away_score);

        return self;
    }


}

pub fn spawn_game_thread(rx: Receiver<GameEvent>) -> JoinHandle<i32> {
    return thread::spawn(move || {
        game_loop(rx);
        return 0;
    });
}

fn game_loop(rx: Receiver<GameEvent>) {
    let mut game = GameState::new();
    loop {
        let ev = rx.recv().unwrap_or(GameEvent::EndBroadcast());
        game = match ev {
            GameEvent::Pregame(home, away) => game.pregame(home, away),
            GameEvent::PlayBall() => game.play_ball(),
            GameEvent::PlayEvent(play_event) => game.play_event(play_event),
            GameEvent::InningEnd(inning) => game.inning_end(inning),
            GameEvent::GameEnd() => game.game_end(),
            GameEvent::EndBroadcast() => break
        }
    }
}