//! Seeded RNG wrapper. All randomness in the game flows through here so
//! that a run is fully reproducible from a seed.
//!
//! The stream is ChaCha8. Saves persist the generator's full state (plus
//! the original seed as plain metadata), so the random stream continues
//! exactly where it left off after a load instead of replaying the run's
//! prefix.

use rand::RngExt;
use rand_core::{SeedableRng, TryRng};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use rand_chacha::ChaCha8Rng as ChaCha;

/// Deterministic RNG.
#[derive(Debug, Clone)]
pub struct Rng {
    pub seed: u64,
    inner: ChaCha,
}

impl Serialize for Rng {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // Persist the seed (metadata) and the full inner generator state,
        // so a loaded run continues the same random stream.
        (self.seed, &self.inner).serialize(s)
    }
}

impl<'de> Deserialize<'de> for Rng {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let (seed, inner) = <(u64, ChaCha)>::deserialize(d).map_err(|e: D::Error| e)?;
        Ok(Self { seed, inner })
    }
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            inner: ChaCha::seed_from_u64(seed),
        }
    }

    pub fn random() -> Self {
        Self::new(rand::random())
    }

    pub fn u64(&mut self) -> u64 {
        use rand_core::TryRng as _;
        self.inner.try_next_u64().unwrap()
    }

    pub fn int(&mut self, range: std::ops::Range<u64>) -> u64 {
        self.inner.random_range(range)
    }

    pub fn int_inclusive(&mut self, range: std::ops::RangeInclusive<u64>) -> u64 {
        self.inner.random_range(range)
    }

    pub fn chance(&mut self, pct: u64) -> bool {
        if pct == 0 {
            return false;
        }
        if pct >= 100 {
            return true;
        }
        self.int(0..100) < pct
    }

    pub fn pick<T: Copy>(&mut self, items: &[T]) -> Option<T> {
        if items.is_empty() {
            return None;
        }
        let n = items.len() as u64;
        let i = self.int(0..n) as usize;
        Some(items[i])
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        let n = items.len();
        for i in (1..n).rev() {
            let j = self.int_inclusive(0..=i as u64) as usize;
            items.swap(i, j);
        }
    }
}

/// Trait so tests can inject fakes.
pub trait RngLike {
    fn u64(&mut self) -> u64;
    fn int(&mut self, range: std::ops::Range<u64>) -> u64;
    fn int_inclusive(&mut self, range: std::ops::RangeInclusive<u64>) -> u64;
    fn chance(&mut self, pct: u64) -> bool;
    fn pick<T: Copy>(&mut self, items: &[T]) -> Option<T>;
    fn shuffle<T>(&mut self, items: &mut [T]);
}

impl RngLike for Rng {
 fn u64(&mut self) -> u64 {
        self.u64()
    }
    fn int(&mut self, range: std::ops::Range<u64>) -> u64 {
        self.inner.random_range(range)
    }
    fn int_inclusive(&mut self, range: std::ops::RangeInclusive<u64>) -> u64 {
        self.inner.random_range(range)
    }
    fn chance(&mut self, pct: u64) -> bool {
        if pct == 0 {
            return false;
        }
        if pct >= 100 {
            return true;
        }
        self.int(0..100) < pct
    }
    fn pick<T: Copy>(&mut self, items: &[T]) -> Option<T> {
        if items.is_empty() {
            return None;
        }
        let n = items.len() as u64;
        let i = self.int(0..n) as usize;
        Some(items[i])
    }
    fn shuffle<T>(&mut self, items: &mut [T]) {
        self.shuffle(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = Rng::new(1234);
        let mut b = Rng::new(1234);
        for _ in 0..100 {
            assert_eq!(a.u64(), b.u64());
        }
    }

    #[test]
    fn roundtrip_continues_stream() {
        let mut a = Rng::new(42);
        // Advance past the run-start prefix, then round-trip a.
        let prefix: Vec<u64> = (0..8).map(|_| a.u64()).collect();
        let json = serde_json::to_string(&a).unwrap();

        let loaded: Rng = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.seed, 42, "seed stays available as metadata");
        let mut b = loaded;
        // b sits at the same stream position as a, so their next draws
        // must match, and must not replay the prefix.
        let after: Vec<u64> = (0..16).map(|_| b.u64()).collect();
        let expected: Vec<u64> = (0..16).map(|_| a.u64()).collect();
        assert_eq!(after, expected, "stream must continue where it left off");
        assert_ne!(after, prefix, "load must not replay the run-start prefix");
    }
}
