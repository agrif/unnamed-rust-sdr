use std::ops::{AddAssign, Mul};
use num::Zero;

mod fir;
pub use fir::*;

mod biquad;
pub use biquad::*;

mod derivative;
pub use derivative::*;

mod pll;
pub use pll::*;

pub trait IntoFilter<A> {
    type Filter: Filter<A>;
    fn into_filter(self, rate: f32) -> Self::Filter;
}

// by rights, this should just be FnMut(A) -> A
// but... fn_traits is not yet stable (??!)
pub trait Filter<A> {
    type Output;
    fn apply(&mut self, value: A) -> Self::Output;
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity;

impl<A> IntoFilter<A> for Identity {
    type Filter = Identity;
    fn into_filter(self, _rate: f32) -> Self::Filter {
        Identity
    }
}

impl<A> Filter<A> for Identity {
    type Output = A;
    fn apply(&mut self, value: A) -> Self::Output {
        value
    }
}
