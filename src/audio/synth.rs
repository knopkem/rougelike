//! Procedural synthesis: oscillators, noise, envelopes, pitch ramps.

#![allow(dead_code)]

pub const SAMPLE_RATE: u32 = 44100;

/// Sine wave sample values, 0..len.
pub fn sine(freq: f32, len: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        out.push((2.0 * std::f32::consts::PI * freq * t).sin());
    }
    out
}

/// Square wave.
pub fn square(freq: f32, len: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let v = (2.0 * std::f32::consts::PI * freq * t).sin();
        out.push(if v >= 0.0 { 1.0 } else { -1.0 });
    }
    out
}

/// Saw wave.
pub fn saw(freq: f32, len: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let phase = (freq * t) % 1.0;
        out.push(if phase < 0.5 {
            4.0 * phase - 1.0
        } else {
            4.0 * (1.0 - phase) - 1.0
        });
    }
    out
}

/// White noise.
pub fn noise(len: usize) -> Vec<f32> {
    use rand_core::TryRng;
    let mut rng = rand::rng();
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        let v = (rng.try_next_u32().unwrap_or(0) as f32 / u32::MAX as f32) * 2.0 - 1.0;
        out.push(v);
    }
    out
}

/// ADSR envelope: attack, decay, sustain, release.
pub fn adsr(len: usize, attack: f32, decay: f32, sustain: f32, release: f32) -> Vec<f32> {
    let mut out = Vec::with_capacity(len);
    let a = (attack * SAMPLE_RATE as f32) as usize;
    let d = (decay * SAMPLE_RATE as f32) as usize;
    let r = (release * SAMPLE_RATE as f32) as usize;
    let sustain_len = len.saturating_sub(a + d + r);
    let mut i = 0;
    // Attack
    for _ in 0..a.min(len) {
        let p = i as f32 / a.max(1) as f32;
        out.push(p);
        i += 1;
    }
    // Decay
    for _ in 0..d.min(len.saturating_sub(i)) {
        let p = 1.0 - (i - a) as f32 / d.max(1) as f32;
        out.push(1.0 - (1.0 - sustain) * p);
        i += 1;
    }
    // Sustain
    for _ in 0..sustain_len {
        out.push(sustain);
        i += 1;
    }
    // Release
    for _ in 0..r.min(len.saturating_sub(i)) {
        let p = 1.0 - (i - a - d - sustain_len) as f32 / r.max(1) as f32;
        out.push(sustain * p);
        i += 1;
    }
    while out.len() < len {
        out.push(0.0);
    }
    out
}

/// Apply envelope to samples.
pub fn apply_envelope(samples: &mut [f32], env: &[f32]) {
    let n = samples.len().min(env.len());
    for i in 0..n {
        samples[i] *= env[i];
    }
}

/// Pitch ramp: linearly interpolate frequency from f0 to f1 over len.
pub fn pitch_ramp(f0: f32, f1: f32, len: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(len);
    let mut phase = 0.0f32;
    for i in 0..len {
        let t = i as f32 / len as f32;
        let f = f0 + (f1 - f0) * t;
        phase += 2.0 * std::f32::consts::PI * f / SAMPLE_RATE as f32;
        out.push(phase.sin());
    }
    out
}

/// Low-pass filter (one-pole).
pub fn lowpass(samples: &[f32], rc: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; samples.len()];
    if samples.is_empty() {
        return out;
    }
    let dt = 1.0 / SAMPLE_RATE as f32;
    let rc_val = rc.max(1e-6);
    let coeff = dt / (rc_val + dt);
    let mut last = 0.0f32;
    for (i, &s) in samples.iter().enumerate() {
        last += coeff * (s - last);
        out[i] = last;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_shape() {
        let s = sine(1000.0, 100);
        assert_eq!(s.len(), 100);
        assert!(s[0].abs() < 1.0);
    }

    #[test]
    fn noise_range() {
        let n = noise(1000);
        for v in &n {
            assert!((-1.0..=1.0).contains(v));
        }
    }
}
