use std::time::{SystemTime, UNIX_EPOCH};

/// Simple Xorshift64 random number generator
/// This is a fast, simple PRNG suitable for non-cryptographic purposes
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Create a new RNG with a seed based on current time
    pub fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        Self::with_seed(seed)
    }

    /// Create a new RNG with a specific seed
    pub fn with_seed(seed: u64) -> Self {
        // Ensure seed is never zero (xorshift requirement)
        let state = if seed == 0 { 1 } else { seed };
        Self { state }
    }

    /// Generate the next random u64 using Xorshift64
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Generate a random f32 in the range [0.0, 1.0)
    pub fn gen_f32(&mut self) -> f32 {
        // Use the upper 24 bits for better distribution
        let value = (self.next_u64() >> 40) as f32;
        value / 16777216.0 // 2^24
    }

    /// Generate a random f32 in a specific range [min, max]
    pub fn gen_range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.gen_f32()
    }

    /// Generate a random u32
    #[allow(dead_code)]
    pub fn gen_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    /// Generate a random u32 in a specific range [min, max]
    #[allow(dead_code)]
    pub fn gen_range_u32(&mut self, min: u32, max: u32) -> u32 {
        if max <= min {
            return min;
        }
        let range = max - min + 1;
        min + (self.gen_u32() % range)
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_basic() {
        let mut rng = Rng::new();

        // Test that we can generate numbers
        let _val1 = rng.gen_f32();
        let _val2 = rng.gen_u32();

        // Test that consecutive calls produce different values (usually)
        let a = rng.gen_f32();
        let b = rng.gen_f32();
        assert_ne!(a, b);
    }

    #[test]
    fn test_rng_range_f32() {
        let mut rng = Rng::new();

        for _ in 0..100 {
            let val = rng.gen_range_f32(0.0, 1.0);
            assert!((0.0..=1.0).contains(&val));
        }

        for _ in 0..100 {
            let val = rng.gen_range_f32(5.0, 10.0);
            assert!((5.0..=10.0).contains(&val));
        }
    }

    #[test]
    fn test_rng_deterministic() {
        let mut rng1 = Rng::with_seed(12345);
        let mut rng2 = Rng::with_seed(12345);

        // Same seed should produce same sequence
        for _ in 0..10 {
            assert_eq!(rng1.gen_f32(), rng2.gen_f32());
        }
    }
}
