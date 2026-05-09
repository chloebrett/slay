pub trait Rng {
    fn shuffle<T>(&mut self, slice: &mut [T]);
}

pub struct SeededRng {
    seed: u64,
    rng: rand::rngs::StdRng,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        use rand::SeedableRng;
        Self { seed, rng: rand::rngs::StdRng::seed_from_u64(seed) }
    }

    pub fn seed(&self) -> u64 { self.seed }
}

impl Rng for SeededRng {
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        use rand::seq::SliceRandom;
        slice.shuffle(&mut self.rng);
    }
}

pub struct NoOpRng;

impl Rng for NoOpRng {
    fn shuffle<T>(&mut self, _: &mut [T]) {}
}

pub struct ThreadRng(rand::rngs::ThreadRng);

impl ThreadRng {
    pub fn new() -> Self {
        Self(rand::thread_rng())
    }
}

impl Default for ThreadRng {
    fn default() -> Self {
        Self::new()
    }
}

impl Rng for ThreadRng {
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        use rand::seq::SliceRandom;
        slice.shuffle(&mut self.0);
    }
}

pub enum AnyRng {
    Thread(ThreadRng),
    Seeded(Box<SeededRng>),
    NoOp(NoOpRng),
}

impl AnyRng {
    pub fn seeded(seed: u64) -> Self {
        AnyRng::Seeded(Box::new(SeededRng::new(seed)))
    }

    pub fn seed(&self) -> Option<u64> {
        match self {
            AnyRng::Seeded(r) => Some(r.seed()),
            _ => None,
        }
    }
}

impl Rng for AnyRng {
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        match self {
            AnyRng::Thread(r)  => r.shuffle(slice),
            AnyRng::Seeded(r)  => r.shuffle(slice),
            AnyRng::NoOp(r)    => r.shuffle(slice),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_rng_noop_does_not_shuffle() {
        let mut v = vec![1, 2, 3, 4, 5];
        let mut rng = AnyRng::NoOp(NoOpRng);
        rng.shuffle(&mut v);
        assert_eq!(v, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn any_rng_thread_can_shuffle() {
        let mut v = vec![0u8; 1];
        let mut rng = AnyRng::Thread(ThreadRng::new());
        rng.shuffle(&mut v); // just verify it doesn't panic
    }

    #[test]
    fn seeded_rng_same_seed_produces_same_shuffle() {
        let mut v1 = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        AnyRng::seeded(42).shuffle(&mut v1);

        let mut v2 = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        AnyRng::seeded(42).shuffle(&mut v2);

        assert_eq!(v1, v2);
    }

    #[test]
    fn seeded_rng_different_seeds_produce_different_shuffles() {
        let mut v1 = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        AnyRng::seeded(1).shuffle(&mut v1);

        let mut v2 = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        AnyRng::seeded(2).shuffle(&mut v2);

        assert_ne!(v1, v2);
    }

    #[test]
    fn seeded_rng_exposes_its_seed() {
        let rng = AnyRng::seeded(99);
        assert_eq!(rng.seed(), Some(99));
    }

    #[test]
    fn non_seeded_rng_has_no_seed() {
        assert_eq!(AnyRng::NoOp(NoOpRng).seed(), None);
    }
}
