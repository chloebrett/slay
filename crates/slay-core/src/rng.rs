pub trait Rng {
    fn shuffle<T>(&mut self, slice: &mut [T]);
    fn gen_bool(&mut self, probability: f64) -> bool;

    fn choose<T: Copy>(&mut self, slice: &mut [T]) -> T {
        self.shuffle(slice);
        slice[0]
    }
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

    fn gen_bool(&mut self, probability: f64) -> bool {
        use rand::Rng as _;
        self.rng.gen_bool(probability)
    }
}

pub struct NoOpRng;

impl Rng for NoOpRng {
    fn shuffle<T>(&mut self, _: &mut [T]) {}
    fn gen_bool(&mut self, _: f64) -> bool { true }
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

    fn gen_bool(&mut self, probability: f64) -> bool {
        use rand::Rng as _;
        self.0.gen_bool(probability)
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

    fn gen_bool(&mut self, probability: f64) -> bool {
        match self {
            AnyRng::Thread(r)  => r.gen_bool(probability),
            AnyRng::Seeded(r)  => r.gen_bool(probability),
            AnyRng::NoOp(r)    => r.gen_bool(probability),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_bool_noop_always_returns_true() {
        let mut rng = AnyRng::NoOp(NoOpRng);
        assert!(rng.gen_bool(0.0), "NoOpRng should always return true");
    }

    #[test]
    fn gen_bool_seeded_gives_correct_rate() {
        let mut rng = AnyRng::seeded(42);
        let trials = 10_000;
        let hits = (0..trials).filter(|_| rng.gen_bool(0.40)).count();
        let rate = hits as f64 / trials as f64;
        assert!((rate - 0.40).abs() < 0.02, "expected ~40% but got {rate:.2}");
    }

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
