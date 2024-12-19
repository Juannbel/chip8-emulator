extern crate sdl2;

use sdl2::audio::{AudioCallback, AudioSpecDesired};

const FREQUENCY: f32 = 440.0;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct Speaker {
    device: sdl2::audio::AudioDevice<SquareWave>,
}

impl Speaker {
    pub fn new(audio_subsystem: sdl2::AudioSubsystem) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| SquareWave {
                phase_inc: FREQUENCY / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            })
            .unwrap();

        device.pause();

        Self { device }
    }

    pub fn start(&self) {
        self.device.resume();
    }

    pub fn stop(&self) {
        self.device.pause();
    }
}
