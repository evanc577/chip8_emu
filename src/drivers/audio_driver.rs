use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

pub struct AudioDriver {
    device: AudioDevice<SquareWave>,
}

impl AudioDriver {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: None,
            channels: Some(1), // mono
            samples: None,     // default sample size
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                // initialize the audio callback
                SquareWave {
                    phase_inc: 240.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.5,
                }
            })
            .unwrap();

        AudioDriver { device }
    }

    pub fn beep(&self, beep: bool) {
        if beep {
            self.device.resume();
        } else {
            self.device.pause();
        }
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = self.volume * if self.phase < 0.5 { 1.0 } else { -1.0 };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
