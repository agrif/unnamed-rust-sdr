use crate::Signal;

mod convolve;
pub use convolve::*;

mod simple;
pub use simple::*;

mod fir;
pub use fir::*;

mod biquad;
pub use biquad::*;

mod derivative;
pub use derivative::*;

mod pll;
pub use pll::*;

// by rights, this should just be FnMut(A) -> A
// but... fn_traits is not yet stable (??!)
pub trait Filter<A> {
    type Output;
    fn apply(&mut self, value: A) -> Self::Output;
}

pub trait FilterDesign<A>: Sized {
    type Output;
    type Filter: Filter<A, Output=Self::Output>;
    fn design(self, rate: f32) -> Self::Filter;

    fn design_for<S>(self, signal: &S) -> Self::Filter
    where
        S: Signal<Sample=A>
    {
        self.design(signal.rate())
    }
}
