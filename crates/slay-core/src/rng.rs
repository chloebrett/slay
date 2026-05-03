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

impl Rng for ThreadRng {
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        use rand::seq::SliceRandom;
        slice.shuffle(&mut self.0);
    }
}
