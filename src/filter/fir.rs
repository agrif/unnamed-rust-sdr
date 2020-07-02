use super::{Filter, FilterDesign};
use super::convolve::Convolve;

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Fir<C, A> {
    coef: Vec<C>,
    buffer: VecDeque<A>,
}

impl<C, A> Fir<C, A> where A: Convolve<C> {
    pub fn new(coef: Vec<C>) -> Self {
        Fir {
            buffer: std::iter::repeat(A::zero()).take(coef.len()).collect(),
            coef,
        }
    }
}

impl<C, A> Filter<A> for Fir<C, A> where A: Convolve<C> {
    type Output = A;
    fn apply(&mut self, value: A) -> Self::Output {
        self.buffer.pop_back();
        self.buffer.push_front(value);

        let mut accum = A::zero();
        for (c, v) in self.coef.iter().zip(self.buffer.iter()) {
            accum.accumulate(v, c);
        }
        accum
    }
}


impl<C, A> FilterDesign<A> for Fir<C, A> where A: Convolve<C> {
    type Output = A;
    type Filter = Fir<C, A>;
    fn design(self, _rate: f32) -> Self::Filter {
        Fir::new(self.coef)
    }
}

impl<C, A> FilterDesign<A> for Vec<C> where A: Convolve<C> {
    type Output = A;
    type Filter = Fir<C, A>;
    fn design(self, _rate: f32) -> Self::Filter {
        Fir::new(self)
    }
}

impl<'a, C, A> FilterDesign<A> for &'a [C] where A: Convolve<C>, C: Clone {
    type Output = A;
    type Filter = Fir<C, A>;
    fn design(self, _rate: f32) -> Self::Filter {
        Fir::new(self.to_owned())
    }
}
