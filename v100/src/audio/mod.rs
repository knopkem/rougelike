//! Audio: procedurally synthesized sound effects (no audio assets).
//!
//! The audio engine is optional (behind the `audio` feature flag, enabled by
//! default). When disabled, `AudioEngine` is a no-op so the core and UI work
//! identically in headless/CI environments.
//!
//! All sounds are synthesized at runtime from simple waveforms — no audio
//! files are shipped.

pub mod sfx;
pub mod synth;

/// The audio engine. When the `audio` feature is disabled, this is a no-op.
#[derive(Debug)]
pub struct AudioEngine {
    #[cfg(feature = "audio")]
    inner: synth::SynthContext,
    #[cfg(not(feature = "audio"))]
    _disabled: (),
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEngine {
    /// Create a new audio engine. If no audio device is available, the engine
    /// is created but playback is a no-op (the game continues silently).
    pub fn new() -> Self {
        #[cfg(feature = "audio")]
        {
            Self {
                inner: synth::SynthContext::new(),
            }
        }
        #[cfg(not(feature = "audio"))]
        {
            Self { _disabled: () }
        }
    }

    /// Play a sound effect.
    pub fn play(&mut self, sfx: sfx::Sfx) {
        #[cfg(feature = "audio")]
        {
            self.inner.play(sfx);
        }
        #[cfg(not(feature = "audio"))]
        {
            let _ = sfx;
        }
    }

    /// Whether audio is enabled and a device is available.
    pub fn is_enabled(&self) -> bool {
        #[cfg(feature = "audio")]
        {
            self.inner.is_available()
        }
        #[cfg(not(feature = "audio"))]
        {
            false
        }
    }
}
