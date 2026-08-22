//! Seeded RNG wrapper. All randomness in the game flows through this type so a
//! run can be reproduced exactly from a seed (`--seed`). The state is
//! serializable, so it is saved with the game for deterministic save/load.

use rand::distr::uniform::SampleUniform;
use rand::rngs::SysRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha12Rng;
use rand_core::TryRng;
use serde::{Deserialize, Serialize};

/// A deterministic, save-safe RNG backed by ChaCha12.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rng(ChaCha12Rng);

impl Rng {
    /// Create an RNG from an explicit seed (reproducible runs).
    pub fn new(seed: u64) -> Self {
        Self(ChaCha12Rng::seed_from_u64(seed))
    }

    /// Create an RNG seeded from the OS entropy source.
    pub fn random() -> Self {
        let mut seed = [0u8; 32];
        SysRng
            .try_fill_bytes(&mut seed)
            .expect("OS entropy source available");
        Self(ChaCha12Rng::from_seed(seed))
    }

    /// Uniform random value in `low..high` (exclusive high).
    pub fn range<T: SampleUniform + PartialOrd>(&mut self, low: T, high: T) -> T {
        self.0.random_range(low..high)
    }

    /// True with probability `pct` percent (0..=100).
    pub fn chance(&mut self, pct: u32) -> bool {
        if pct >= 100 {
            return true;
        }
        if pct == 0 {
            return false;
        }
        self.0.random_range(0..100u32) < pct
    }

    /// Pick a random element from a non-empty slice.
    pub fn pick<T: Clone>(&mut self, items: &[T]) -> Option<T> {
        if items.is_empty() {
            return None;
        }
        let idx = self.0.random_range(0..items.len());
        Some(items[idx].clone())
    }

    /// Shuffle a slice in place (Fisher-Yates).
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        items.shuffle(&mut self.0);
    }

    /// Weighted pick: `weights` must be the same length as `items`.
    pub fn weighted<T: Clone>(&mut self, items: &[T], weights: &[u32]) -> Option<T> {
        if items.is_empty() || items.len() != weights.len() {
            return None;
        }
        let total: u32 = weights.iter().sum();
        if total == 0 {
            return None;
        }
        let mut roll = self.0.random_range(0..total);
        for (item, w) in items.iter().zip(weights.iter()) {
            if roll < *w {
                return Some(item.clone());
            }
            roll -= *w;
        }
        Some(items.last()?.clone())
    }

    /// Access the underlying RNG for advanced distributions.
    pub fn inner(&mut self) -> &mut ChaCha12Rng {
        &mut self.0
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::random()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_from_seed() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.range(0, 1000), b.range(0, 1000));
        }
    }

    #[test]
    fn range_is_in_bounds() {
        let mut r = Rng::new(1);
        for _ in 0..1000 {
            let v = r.range(5, 10);
            assert!((5..10).contains(&v));
        }
    }

    #[test]
    fn chance_respects_extremes() {
        let mut r = Rng::new(1);
        assert!(r.chance(100));
        assert!(!r.chance(0));
    }

    #[test]
    fn pick_empty_is_none() {
        let mut r = Rng::new(1);
        assert!(r.pick::<u32>(&[]).is_none());
    }

    #[test]
    fn weighted_respects_zero() {
        let mut r = Rng::new(1);
        assert!(r.weighted(&[1, 2], &[0, 0]).is_none());
        let v = r.weighted(&["a", "b"], &[0, 100]).unwrap();
        assert_eq!(v, "b");
    }
}
