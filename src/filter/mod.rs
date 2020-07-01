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

#[derive(Clone, Debug)]
pub struct Monitor<F>(pub f32, pub F);

#[derive(Clone, Debug)]
pub struct Monitored<F> {
    every: usize,
    i: usize,
    func: F,
}

impl<F, A> IntoFilter<A> for Monitor<F> where F: FnMut(&A) -> () {
    type Filter = Monitored<F>;
    fn into_filter(self, rate: f32) -> Self::Filter {
        Monitored {
            every: (rate / self.0).round() as usize,
            i: 0,
            func: self.1,
        }
    }
}

impl<F, A> Filter<A> for Monitored<F> where F: FnMut(&A) -> () {
    type Output = A;
    fn apply(&mut self, value: A) -> Self::Output {
        self.i += 1;
        if self.i >= self.every {
            self.i = 0;
            (self.func)(&value);
        }
        value
    }
}
