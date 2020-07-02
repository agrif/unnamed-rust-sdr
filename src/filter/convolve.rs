use std::ops::{AddAssign, Mul};
use num::Zero;

pub trait Convolve<C>: Clone + Zero {
    fn accumulate(&mut self, a: &Self, c: &C);
}

impl<C, A> Convolve<C> for A
where
    C: Clone,
    A: Clone + Zero + AddAssign<A> + Mul<C, Output=A>,
{
    fn accumulate(&mut self, a: &Self, c: &C) {
        *self += a.clone() * c.clone();
    }
}
