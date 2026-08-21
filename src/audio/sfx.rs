//! SFX engine: routes game events to procedurally synthesized sounds.
//!
//! Backend chain: rodio playback (opened lazily on a worker thread) with
//! terminal-BEL fallback for critical cues when no device is available.
//! `SfxEngine::disabled()` produces no output at all (used by `--no-audio`).

#[cfg(feature = "audio")]
use std::sync::mpsc;

use crate::audio::synth;
use crate::core::events::GameEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    Footstep,
    Hit,
    Miss,
    Crit,
    MonsterDeath,
    PlayerDeath,
    Pickup,
    Equip,
    Quaff,
    Eat,
    WandCast,
    PotionSplash,
    ScrollRead,
    LevelUp,
    Stairs,
    Door,
    Trap,
    Quest,
    Victory,
    Coin,
    Teleport,
}

impl Sfx {
    pub fn from_event(ev: &GameEvent) -> Option<Sfx> {
        match ev {
            GameEvent::Footstep => Some(Sfx::Footstep),
            GameEvent::Hit { crit } => {
                if *crit {
                    Some(Sfx::Crit)
                } else {
                    Some(Sfx::Hit)
                }
            }
            GameEvent::Miss => Some(Sfx::Miss),
            GameEvent::MonsterDeath { .. } => Some(Sfx::MonsterDeath),
            GameEvent::PlayerDeath => Some(Sfx::PlayerDeath),
            GameEvent::Pickup => Some(Sfx::Pickup),
            GameEvent::Drop => Some(Sfx::Pickup),
            GameEvent::Equip => Some(Sfx::Equip),
            GameEvent::Quaff => Some(Sfx::Quaff),
            GameEvent::Eat => Some(Sfx::Eat),
            GameEvent::WandCast { .. } => Some(Sfx::WandCast),
            GameEvent::PotionSplash => Some(Sfx::PotionSplash),
            GameEvent::ScrollRead => Some(Sfx::ScrollRead),
            GameEvent::LevelUp => Some(Sfx::LevelUp),
            GameEvent::Stairs => Some(Sfx::Stairs),
            GameEvent::Door { .. } => Some(Sfx::Door),
            GameEvent::Trap => Some(Sfx::Trap),
            GameEvent::Quest { .. } => Some(Sfx::Quest),
            GameEvent::Victory => Some(Sfx::Victory),
            GameEvent::Coin => Some(Sfx::Coin),
            GameEvent::Teleport => Some(Sfx::Teleport),
            _ => None,
        }
    }

    /// Notes for the SFX: (frequency Hz, duration seconds).
    #[cfg_attr(not(feature = "audio"), allow(dead_code))]
    fn notes(sfx: Sfx) -> &'static [(f32, f32)] {
        match sfx {
            Sfx::Footstep => &[(95.0, 0.07)],
            Sfx::Hit => &[(140.0, 0.08)],
            Sfx::Miss => &[(70.0, 0.1)],
            Sfx::Crit => &[(150.0, 0.09), (230.0, 0.09)],
            Sfx::MonsterDeath => &[(240.0, 0.2)],
            Sfx::PlayerDeath => &[(90.0, 0.7)],
            Sfx::Pickup => &[(880.0, 0.05)],
            Sfx::Equip => &[(400.0, 0.07), (520.0, 0.07)],
            Sfx::Quaff => &[(620.0, 0.09), (780.0, 0.09)],
            Sfx::Eat => &[(300.0, 0.05), (280.0, 0.05), (260.0, 0.05)],
            Sfx::WandCast => &[(180.0, 0.18)],
            Sfx::PotionSplash => &[(500.0, 0.08)],
            Sfx::ScrollRead => &[(1050.0, 0.12), (1400.0, 0.12)],
            Sfx::LevelUp => &[(440.0, 0.08), (554.0, 0.08), (660.0, 0.08), (880.0, 0.14)],
            Sfx::Stairs => &[(262.0, 0.3), (330.0, 0.3)],
            Sfx::Door => &[(160.0, 0.14)],
            Sfx::Trap => &[(110.0, 0.24)],
            Sfx::Quest => &[(392.0, 0.09), (494.0, 0.09), (587.0, 0.16)],
            Sfx::Victory => &[(523.0, 0.09), (659.0, 0.09), (784.0, 0.09), (1047.0, 0.22)],
            Sfx::Coin => &[(988.0, 0.05), (1319.0, 0.09)],
            Sfx::Teleport => &[(300.0, 0.25)],
        }
    }
}

enum Backend {
    /// No rodio (feature off, init in flight, or it failed): BEL for critical cues.
    Bell,
    /// No output at all (e.g. `--no-audio`).
    Disabled,
    #[cfg(feature = "audio")]
    Rodio {
        sink: rodio::stream::MixerDeviceSink,
        mixer: rodio::mixer::Mixer,
    },
}

impl Backend {
    fn label(&self) -> &'static str {
        match self {
            Backend::Bell => "Bell",
            Backend::Disabled => "Disabled",
            #[cfg(feature = "audio")]
            Backend::Rodio { .. } => "Rodio",
        }
    }
}

/// SFX engine: one instance per app; game events are routed through
/// [`SfxEngine::play_event`]. Muting suppresses all output.
pub struct SfxEngine {
    muted: bool,
    backend: Backend,
    #[cfg(feature = "audio")]
    pending: Option<mpsc::Receiver<rodio::stream::MixerDeviceSink>>,
}

impl std::fmt::Debug for SfxEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SfxEngine")
            .field("muted", &self.muted)
            .field("backend", &self.backend.label())
            .finish_non_exhaustive()
    }
}

impl SfxEngine {
    /// Create an engine with full playback. Rodio is opened lazily on a
    /// worker thread so device probing never blocks the game loop; until it
    /// is ready (or if it fails) critical cues fall back to the terminal
    /// BEL.
    pub fn new() -> Self {
        #[cfg(feature = "audio")]
        {
            let (tx, rx) = mpsc::channel();
            let _ = std::thread::Builder::new()
                .name("deepdelve-sfx-init".into())
                .spawn(move || {
                    if let Ok(sink) = rodio::stream::DeviceSinkBuilder::open_default_sink() {
                        let _ = tx.send(sink);
                    }
                });
            Self {
                muted: false,
                backend: Backend::Bell,
                pending: Some(rx),
            }
        }
        #[cfg(not(feature = "audio"))]
        {
            Self {
                muted: false,
                backend: Backend::Bell,
            }
        }
    }

    /// Create an engine that produces no output. No audio device is opened
    /// and no BEL is rung; this is the seam for `--no-audio`.
    pub fn disabled() -> Self {
        Self {
            muted: false,
            backend: Backend::Disabled,
            #[cfg(feature = "audio")]
            pending: None,
        }
    }

    /// Toggle mute; returns the new state.
    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        self.muted
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Play an SFX (no-op if muted or disabled).
    pub fn play(&mut self, sfx: Sfx) {
        if self.muted {
            return;
        }
        if let Backend::Disabled = self.backend {
            return;
        }
        #[cfg(feature = "audio")]
        self.poll_pending();

        match &self.backend {
            Backend::Bell => {
                // Terminal BEL for critical cues.
                if matches!(sfx, Sfx::PlayerDeath | Sfx::Victory | Sfx::Crit) {
                    print!("\x07");
                }
            }
            Backend::Disabled => unreachable!(),
            #[cfg(feature = "audio")]
            Backend::Rodio { mixer, .. } => {
                let channels = std::num::NonZero::new(1u16).expect("mono is non-zero");
                let rate = std::num::NonZero::new(synth::SAMPLE_RATE).expect("non-zero");
                let buffer = rodio::buffer::SamplesBuffer::new(channels, rate, Self::render(sfx));
                mixer.add(buffer);
            }
        }
    }

    pub fn play_event(&mut self, ev: &GameEvent) {
        if let Some(sfx) = Sfx::from_event(ev) {
            self.play(sfx);
        }
    }

    #[cfg(feature = "audio")]
    fn poll_pending(&mut self) {
        let Some(rx) = &self.pending else {
            return;
        };
        let Ok(sink) = rx.try_recv() else {
            return;
        };
        self.backend = Backend::Rodio {
            mixer: sink.mixer().clone(),
            sink,
        };
    }

    /// Synthesize the samples for an SFX: per-note oscillator with an ADSR
    /// envelope, low-pass filtered and scaled to a safe amplitude.
    #[cfg_attr(not(feature = "audio"), allow(dead_code))]
    fn render(sfx: Sfx) -> Vec<f32> {
        let notes = Sfx::notes(sfx);
        let mut total = 0usize;
        for &(_, dur) in notes {
            total += (dur * synth::SAMPLE_RATE as f32) as usize + 1;
        }
        let mut samples = vec![0.0f32; total];
        let mut offset = 0;
        for &(freq, dur) in notes {
            let len = (dur * synth::SAMPLE_RATE as f32) as usize + 1;
            let osc = synth::sine(freq, len);
            let env = synth::adsr(len, 0.004, 0.02, 0.5, 0.03);
            for (i, (o, e)) in osc.iter().zip(env.iter()).enumerate() {
                samples[offset + i] += o * e;
            }
            offset += len;
        }
        let filtered = synth::lowpass(&samples, 0.0004);
        filtered.iter().map(|s| s * 0.25).collect()
    }
}

impl Default for SfxEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SfxEngine {
    fn drop(&mut self) {
        // Quiet the rodio sink teardown message; this is normal app exit.
        #[cfg(feature = "audio")]
        if let Backend::Rodio { sink, .. } = &mut self.backend {
            sink.log_on_drop(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_engine_produces_no_output() {
        let mut e = SfxEngine::disabled();
        // Must not panic or print; nothing else to assert on the no-op path.
        e.play_event(&GameEvent::LevelUp);
        e.play(Sfx::Crit);
        assert!(!e.is_muted());
    }

    #[test]
    fn disabled_engine_is_not_muted_but_silent() {
        let mut e = SfxEngine::disabled();
        assert!(!e.is_muted());
        e.toggle_mute();
        assert!(e.is_muted());
        e.play(Sfx::Hit);
    }

    #[test]
    fn mute_gates_playback() {
        let mut e = SfxEngine::disabled();
        e.toggle_mute();
        assert!(e.is_muted());
        e.play(Sfx::PlayerDeath);
        e.toggle_mute();
        assert!(!e.is_muted());
        e.play(Sfx::PlayerDeath);
    }

    #[test]
    fn render_shapes_are_reasonable() {
        for sfx in [
            Sfx::Hit,
            Sfx::Pickup,
            Sfx::LevelUp,
            Sfx::Stairs,
            Sfx::Footstep,
            Sfx::WandCast,
            Sfx::Teleport,
        ] {
            let s = SfxEngine::render(sfx);
            assert!(!s.is_empty(), "{sfx:?}");
            assert!(s.iter().all(|v| (-1.0..=1.0).contains(v)), "{sfx:?}");
        }
    }

    #[test]
    fn from_event_maps_representatives() {
        assert_eq!(
            Sfx::from_event(&GameEvent::Hit { crit: true }),
            Some(Sfx::Crit)
        );
        assert_eq!(Sfx::from_event(&GameEvent::Pickup), Some(Sfx::Pickup));
        assert_eq!(Sfx::from_event(&GameEvent::LevelUp), Some(Sfx::LevelUp));
        assert_eq!(Sfx::from_event(&GameEvent::Stairs), Some(Sfx::Stairs));
        assert_eq!(Sfx::from_event(&GameEvent::Poisoned), None);
    }
}
