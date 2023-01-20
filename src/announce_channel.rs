use std::sync::mpsc::{Receiver};

use crate::{types::Samp, tts::Speaker};

pub enum AnnounceEvent {
    Beat(), // Short delay.
    Thwack(f32), // Baseball bat sound
    Message(String), // TTS message
    Delay(u64), // Delay, in samples
    Finish() // Stop broadcasting
}

const BEAT_LENGTH: u64 = 1024; // ~46ms @ 22050 Hz
const ANNOUNCE_VOLUME: f32 = 1.0;

enum ChannelState {
    Waiting,
    Sampling,
    Announcing,
    Idle,
    Finished
}

pub struct AnnounceChannel {
    state: ChannelState,
    wait_left: u64,
    volume: f32,
    rx: Receiver<AnnounceEvent>,
    speaker: Speaker,

}

impl AnnounceChannel {

    pub fn new(rx: Receiver<AnnounceEvent>) -> AnnounceChannel {
        return AnnounceChannel {
            state: ChannelState::Idle,
            wait_left: 0,
            volume: 1.0,
            rx: rx,
            speaker: Speaker::new()
        };
    }

    pub fn next(&mut self, buf: &mut [Samp]) -> usize {
        let mut samples_filled: usize = 0;
        let sample_target: usize = buf.len() as usize;

        while samples_filled < sample_target {
            match self.state {
                ChannelState::Waiting => samples_filled += self.wait(&mut buf[samples_filled..]),
                ChannelState::Announcing => samples_filled += self.announce(&mut buf[samples_filled..]),
                ChannelState::Idle => samples_filled += self.idle(&mut buf[samples_filled..]),
                ChannelState::Sampling => samples_filled += self.sample(&mut buf[samples_filled..]),
                ChannelState::Finished => break
            }
        }
        return samples_filled;
    }

    fn wait(&mut self, buf: &mut [Samp]) -> usize {
        if self.wait_left > (buf.len() as u64) {
            buf.fill(0.0);
            self.wait_left -= buf.len() as u64;
            return buf.len();
        } else {
            buf[..(self.wait_left as usize)].fill(0.0);
            let filled = self.wait_left as usize;
            self.wait_left = 0;
            self.get_next_state();
            return filled;
        }
    }

    fn announce(&mut self, buf: &mut [Samp]) -> usize {
        if self.speaker.is_speaking() {
            return self.speaker.next(buf);
        } else {
            self.get_next_state();
            return 0;
        }
    }

    fn idle(&mut self, buf: &mut [Samp]) -> usize {
        if matches!(self.state, ChannelState::Idle) {
            self.get_next_state();
            if !matches!(self.state, ChannelState::Idle) {
                return 0;
            }
        }
        buf.fill(0.0);
        return buf.len();
    }

    fn sample(&mut self, buf: &mut [Samp]) -> usize {
        self.get_next_state();
        return 0;
    }

    fn get_next_state(&mut self) {
        /*
        self.state = self.rx.try_recv().map_or(ChannelState::Idle, |ev| {
            */
        self.state = self.rx.recv().map_or(ChannelState::Finished, |ev|{
            return match ev {
                AnnounceEvent::Beat() => { self.wait_left = BEAT_LENGTH; ChannelState::Waiting },
                AnnounceEvent::Thwack(t) => { self.volume = t; ChannelState::Sampling },
                AnnounceEvent::Delay(d) => { self.wait_left = d; ChannelState::Waiting },
                AnnounceEvent::Message(s) => { self.speaker.say(&s); ChannelState::Announcing },
                AnnounceEvent::Finish() => ChannelState::Finished
            }
        });

    }

}