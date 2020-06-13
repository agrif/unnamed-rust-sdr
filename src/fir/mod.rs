use std::collections::VecDeque;
use std::ops::{AddAssign, Mul};
use num::Zero;

pub trait IntoFir<C> {
    fn into_fir<A>(self, rate: f64) -> Fir<C, A> where A: Convolve<C>;
}

pub trait Convolve<C>: Zero {
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

#[derive(Clone, Debug)]
pub struct Fir<C, A> {
    coef: Vec<C>,
    buffer: VecDeque<A>,
}

impl<C, A> Fir<C, A> where A: Convolve<C> {
    pub fn new(coef: Vec<C>) -> Self {
        Fir {
            buffer: VecDeque::with_capacity(coef.len()),
            coef,
        }
    }

    pub fn apply(&mut self, value: A) -> Option<A> {
        self.buffer.push_back(value);
        if self.buffer.len() >= self.coef.len() {
            // we have enough data to do a convolution
            let mut accum: A = Zero::zero();
            for (c, v) in self.coef.iter().zip(self.buffer.iter()) {
                accum.accumulate(v, c);
            }
            self.buffer.pop_front();
            Some(accum)
        } else {
            // need more data
            None
        }
    }
}

impl<C> IntoFir<C> for Vec<C> {
    fn into_fir<A>(self, _rate: f64) -> Fir<C, A> where A: Convolve<C> {
        Fir::new(self)
    }
}

impl<'a, C> IntoFir<C> for &'a [C] where C: Clone {
    fn into_fir<A>(self, _rate: f64) -> Fir<C, A> where A: Convolve<C> {
        Fir::new(self.to_owned())
    }
}
