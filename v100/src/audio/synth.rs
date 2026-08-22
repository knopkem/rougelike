//! Procedural sound synthesis. Each `Sfx` is rendered to a short PCM buffer
//! from simple waveforms (sine, square, noise) with an exponential decay
//! envelope. No audio files are used.
//!
//! This module is only compiled when the `audio` feature is enabled.

#[cfg(feature = "audio")]
mod impl_ {
    use crate::audio::sfx::Sfx;
    use rodio::Player;
    use rodio::buffer::SamplesBuffer;
    use rodio::stream::DeviceSinkBuilder;
    use std::num::NonZero;

    /// The sample rate for synthesized audio.
    const SAMPLE_RATE: u32 = 44100;

    /// A context that owns the audio output sink and can play synthesized
    /// buffers. If no audio device is available, the sink is `None` and
    /// playback is a no-op (the game continues silently).
    pub struct SynthContext {
        sink: Option<rodio::stream::MixerDeviceSink>,
    }

    impl std::fmt::Debug for SynthContext {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("SynthContext").finish_non_exhaustive()
        }
    }

    impl SynthContext {
        /// Create a new synthesis context. If no audio device is available, the
        /// context is created with no sink and playback is a no-op.
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            match DeviceSinkBuilder::open_default_sink() {
                Ok(sink) => Self { sink: Some(sink) },
                Err(_) => Self { sink: None },
            }
        }

        /// Whether an audio device is available.
        pub fn is_available(&self) -> bool {
            self.sink.is_some()
        }

        /// Play a sound effect by synthesizing it on the fly.
        pub fn play(&mut self, sfx: Sfx) {
            let sink = match &self.sink {
                Some(s) => s,
                None => return,
            };
            let samples = synthesize(sfx);
            if samples.is_empty() {
                return;
            }
            let channels = NonZero::new(1u16).expect("1 is non-zero");
            let rate = NonZero::new(SAMPLE_RATE).expect("44100 is non-zero");
            let buffer = SamplesBuffer::new(channels, rate, samples);
            // Connect a player to the mixer, append the buffer, and drop the
            // player. The source is queued into the mixer's output, so it
            // keeps playing after the player is dropped.
            let mixer = sink.mixer();
            let player = Player::connect_new(mixer);
            player.append(buffer);
            drop(player);
        }
    }

    /// Synthesize the samples for a given sound effect.
    fn synthesize(sfx: Sfx) -> Vec<f32> {
        let (freq, duration, wave, decay) = params(sfx);
        let n = (duration * SAMPLE_RATE as f32) as usize;
        let mut out = Vec::with_capacity(n);
        let two_pi = std::f32::consts::PI * 2.0;
        let mut phase = 0.0f32;
        let mut noise_state = 0.5f32;
        for i in 0..n {
            let t = i as f32 / n as f32;
            // Exponential decay envelope.
            let env = (1.0 - t).powf(decay);
            let sample = match wave {
                Wave::Sine => (phase * two_pi).sin(),
                Wave::Square => {
                    if (phase * two_pi).sin() >= 0.0 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Wave::Noise => {
                    // Simple linear congruential noise.
                    noise_state =
                        (noise_state * 1664525.0 + 1013904223.0) % 4294967296.0 / 4294967296.0;
                    noise_state * 2.0 - 1.0
                }
            };
            phase = (phase + freq / SAMPLE_RATE as f32) % 1.0;
            out.push(sample * env * 0.3);
        }
        out
    }

    /// The waveform type.
    #[derive(Debug, Clone, Copy)]
    enum Wave {
        Sine,
        Square,
        Noise,
    }

    /// Parameters for each sound effect: (frequency Hz, duration s, waveform,
    /// decay exponent).
    fn params(sfx: Sfx) -> (f32, f32, Wave, f32) {
        match sfx {
            Sfx::Step => (180.0, 0.05, Wave::Sine, 4.0),
            Sfx::PlayerHit => (150.0, 0.08, Wave::Square, 3.0),
            Sfx::PlayerCrit => (320.0, 0.15, Wave::Square, 2.5),
            Sfx::Miss => (220.0, 0.1, Wave::Sine, 3.0),
            Sfx::MonsterHit => (120.0, 0.08, Wave::Square, 3.0),
            Sfx::MonsterDeath => (90.0, 0.3, Wave::Noise, 2.0),
            Sfx::PlayerDeath => (60.0, 0.8, Wave::Noise, 1.5),
            Sfx::Pickup => (520.0, 0.08, Wave::Sine, 3.0),
            Sfx::Drop => (300.0, 0.08, Wave::Sine, 3.0),
            Sfx::Equip => (440.0, 0.1, Wave::Sine, 3.0),
            Sfx::Quaff => (600.0, 0.12, Wave::Sine, 3.0),
            Sfx::Eat => (250.0, 0.1, Wave::Square, 3.0),
            Sfx::Read => (700.0, 0.1, Wave::Sine, 3.0),
            Sfx::WandFire => (800.0, 0.15, Wave::Noise, 2.5),
            Sfx::LevelUp => (660.0, 0.3, Wave::Sine, 2.0),
            Sfx::StairsDown => (200.0, 0.2, Wave::Sine, 2.5),
            Sfx::StairsUp => (400.0, 0.2, Wave::Sine, 2.5),
            Sfx::Door => (150.0, 0.1, Wave::Square, 3.0),
            Sfx::Trap => (100.0, 0.2, Wave::Noise, 2.5),
            Sfx::QuestComplete => (880.0, 0.4, Wave::Sine, 2.0),
            Sfx::QuestAccepted => (550.0, 0.2, Wave::Sine, 2.5),
            Sfx::Victory => (1046.0, 0.6, Wave::Sine, 1.5),
            Sfx::Teleport => (1200.0, 0.2, Wave::Sine, 3.0),
            Sfx::Coin => (900.0, 0.08, Wave::Sine, 3.0),
            Sfx::MonsterAbility => (400.0, 0.2, Wave::Noise, 2.5),
            Sfx::Blip => (440.0, 0.05, Wave::Sine, 3.0),
        }
    }
}

#[cfg(feature = "audio")]
pub use impl_::SynthContext;
