use crate::filter::FilterDesign;
use crate::filter;
use crate::resample;

mod times;

mod sources;
pub use sources::*;

mod adapters;
pub use adapters::*;

pub trait Signal {
    type Sample;
    fn next(&mut self) -> Option<Self::Sample>;
    fn rate(&self) -> f32;

    fn block(self, size: f32) -> Block<Self>
    where
        Self::Sample: Clone,
        Self: Sized,
    {
        Block::new(self, size)
    }

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

    fn filter<F>(self, filter: F) -> Filter<Self, F::Filter>
    where
        F: FilterDesign<Self::Sample>,
        Self: Sized,
    {
        Filter::new(self, filter)
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

    fn monitor<F>(self, rate: f32, f: F) -> Filter<Self, filter::Monitor<F>>
    where
        F: FnMut(&Self::Sample) -> (),
        Self: Sized,
    {
        self.filter(filter::MonitorD(rate, f))
    }

    fn resample(self, rate: f32) -> Resample<Self>
    where
        Self::Sample: resample::Resample,
        Self: Sized,
    {
        self.resample_with(resample::ConverterType::SincBestQuality, rate)
    }

    fn resample_with(self, typ: resample::ConverterType, rate: f32)
                     -> Resample<Self>
    where
        Self::Sample: resample::Resample,
        Self: Sized,
    {
        Resample::new(self, typ, rate)
    }

    fn skip(self, duration: f32) -> Skip<Self>
    where
        Self: Sized,
    {
        Skip::new(self, duration)
    }

    fn stereo(self) -> Stereo<Self>
    where
        Self: Sized,
    {
        Stereo::new(self)
    }

    fn take(self, duration: f32) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, duration)
    }

    fn tee(self) -> (Tee<Self>, Tee<Self>)
    where
        Self: Sized,
    {
        Tee::new(self)
    }
}

