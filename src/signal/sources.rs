use super::Signal;
use super::times::Times;

use num::Complex;

#[derive(Debug, Clone)]
pub struct FromIter<I> {
    iter: I,
    rate: f32,
}

impl<I> FromIter<I> {
    pub fn new(rate: f32, iter: I) -> Self {
        FromIter {
            iter,
            rate,
        }
    }
}

impl<I> Signal for FromIter<I> where I: Iterator {
    type Sample = I::Item;
    fn next(&mut self) -> Option<Self::Sample> {
        self.iter.next()
    }
    fn rate(&self) -> f32 {
        self.rate
    }
}

pub fn from_iter<I>(rate: f32, iter: I) -> impl Signal<Sample=I::Item>
where
    I: Iterator,
{
    FromIter::new(rate, iter)
}

pub fn from_func<F, A>(rate: f32, mut f: F) -> impl Signal<Sample=A>
where
    F: FnMut(f32) -> A,
{
    from_iter(rate, Times::new(rate).map(move |t| f(t)))
}

pub fn constant<A>(rate: f32, value: A) -> impl Signal<Sample=A>
where
    A: Clone,
{
    from_iter(rate, std::iter::repeat(value))
}

pub fn one<A>(rate: f32) -> impl Signal<Sample=A>
where
    A: num::One + Clone
{
    constant(rate, A::one())
}

pub fn zero<A>(rate: f32) -> impl Signal<Sample=A>
where
    A: num::Zero + Clone
{
    constant(rate, A::zero())
}

pub fn freq(rate: f32, freq: f32, phase: f32)
            -> impl Signal<Sample=Complex<f32>>
{
    use std::f32::consts::PI;
    from_func(rate,
              move |t| Complex::new(0.0, PI * 2.0 * freq * t + phase).exp())
}
