pub trait Rng {
    fn shuffle<T>(&mut self, slice: &mut [T]);
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
    NoOp(NoOpRng),
}

impl Rng for AnyRng {
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        match self {
            AnyRng::Thread(r) => r.shuffle(slice),
            AnyRng::NoOp(r)   => r.shuffle(slice),
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
}
