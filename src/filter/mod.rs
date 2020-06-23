use std::ops::{AddAssign, Mul};
use num::Zero;

mod fir;
pub use fir::*;

mod biquad;
pub use biquad::*;

mod derivative;
pub use derivative::*;

pub trait IntoFilter<A> {
    type Filter: Filter<A>;
    fn into_filter(self, rate: f32) -> Self::Filter;
}

// by rights, this should just be FnMut(A) -> A
// but... fn_traits is not yet stable (??!)
pub trait Filter<A> {
    fn apply(&mut self, value: A) -> A;
}

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
