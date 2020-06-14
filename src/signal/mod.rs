use crate::fir::{Convolve, IntoFir};

pub trait Signal {
    type Sample;
    fn next(&mut self) -> Option<Self::Sample>;
    fn rate(&self) -> f32;

    fn enumerate(self) -> Enumerate<Self> where Self: Sized {
        Enumerate::new(self)
    }

    fn wrap_enumerate<F, I>(self, f: F) -> FromIter<I>
    where
        F: FnOnce(Enumerate<Self>) -> I,
        Self: Sized,
    {
        FromIter::new(self.rate(), f(self.enumerate()))
    }

    fn filter<F, C>(self, fir: F) -> Filter<Self, C>
    where
        F: IntoFir<C>,
        Self::Sample: Convolve<C>,
        Self: Sized,
    {
        Filter::new(self, fir)
    }

    fn iter(self) -> Iter<Self> where Self: Sized {
        Iter::new(self)
    }

    fn wrap<F, I>(self, f: F) -> FromIter<I>
    where
        F: FnOnce(Iter<Self>) -> I,
        Self: Sized,
    {
        FromIter::new(self.rate(), f(self.iter()))
    }

    fn map<F, A>(self, f: F) -> Map<Self, F>
    where
        F: FnMut(Self::Sample) -> A,
        Self: Sized,
    {
        Map::new(self, f)
    }

    fn resample(self, rate: f32) -> Resample<Self>
    where
        Self::Sample: Channels,
        Self: Sized,
    {
        Resample::new(self, rate)
    }

    fn take(self, duration: f32) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, duration)
    }
}

mod times;

mod sources;
pub use sources::*;

mod adapters;
pub use adapters::*;

