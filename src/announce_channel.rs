use std::{sync::mpsc::{Receiver}, iter::zip};

use crate::{types::Samp, tts::Speaker, sample_library::SampleLibrary};

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

pub struct AnnounceChannel<'a> {
    state: ChannelState,
    wait_left: u64,
    rx: Receiver<AnnounceEvent>,
    speaker: Speaker,
    thwacks: &'a SampleLibrary,

    volume: f32,
    current_sample: Option<&'a[Samp]>,
}

impl<'a> AnnounceChannel<'a> {

    pub fn new(rx: Receiver<AnnounceEvent>, thwacks: &'a mut SampleLibrary) -> AnnounceChannel<'a> {
        return AnnounceChannel {
            state: ChannelState::Idle,
            wait_left: 0,
            volume: 1.0,
            rx: rx,
            speaker: Speaker::new(),
            thwacks: thwacks,
            current_sample: None
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
        return match self.current_sample {
            None => {
                self.get_next_state(); 0
            },
            Some(sample) => {
                if buf.len() >= sample.len() {
                    for (s, b) in zip(sample, buf) {
                        *b = *s * self.volume;
                    }
                    self.current_sample = None;
                    sample.len()
                } else {
                    // buf.copy_from_slice(&sample[..buf.len()]);
                    let samples_copied = buf.len();
                    for (s, b) in zip(sample, buf) {
                        *b = *s * self.volume;
                    }
                    self.current_sample = Some(&sample[samples_copied..]);
                    samples_copied
                }
            }
        };
    }

    fn get_next_state(&mut self)
    {
        /*
        self.state = self.rx.try_recv().map_or(ChannelState::Idle, |ev| {
            */
        self.state = self.rx.recv().map_or(ChannelState::Finished, |ev|{
            return match ev {
                AnnounceEvent::Beat() => { self.wait_left = BEAT_LENGTH; ChannelState::Waiting },
                AnnounceEvent::Thwack(t) => {
                    self.current_sample = Some(self.thwacks.get(fastrand::usize(..self.thwacks.len())));
                    self.volume = t; 
                    ChannelState::Sampling
                },
                AnnounceEvent::Delay(d) => { self.wait_left = d; ChannelState::Waiting },
                AnnounceEvent::Message(s) => { self.speaker.say(&s); ChannelState::Announcing },
                AnnounceEvent::Finish() => ChannelState::Finished
            }
        });

    }

}