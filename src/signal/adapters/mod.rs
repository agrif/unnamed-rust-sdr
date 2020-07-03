use crate::Signal;
use super::times::Times;
use crate::filter;

use std::collections::VecDeque;

mod block;
pub use block::*;

mod resample;
pub use resample::*;

#[derive(Debug, Clone)]
pub struct Decimate<S> {
    wait: usize,
    signal: S,
}

impl<S> Decimate<S> where S: Signal {
    pub(super) fn new(signal: S, rate: f32) -> Self {
        Decimate {
            wait: (signal.rate() / rate).round() as usize,
            signal,
        }
    }
}

impl<S> Signal for Decimate<S> where S: Signal {
    type Sample = S::Sample;
    fn next(&mut self) -> Option<Self::Sample> {
        for _ in 0..(self.wait - 1) {
            if let None = self.signal.next() {
                return None;
            }
        }
        self.signal.next()
    }
    fn rate(&self) -> f32 {
        self.signal.rate()
    }
}

#[derive(Debug, Clone)]
pub struct Enumerate<S> {
    signal: S,
    times: Times,
}

impl<S> Enumerate<S> where S: Signal {
    pub(super) fn new(signal: S) -> Self {
        Enumerate {
            times: Times::new(signal.rate()),
            signal,
        }
    }
}

impl<S> Iterator for Enumerate<S> where S: Signal {
    type Item = (f32, S::Sample);
    fn next(&mut self) -> Option<Self::Item> {
        // unwrap is safe: times is infinite
        self.signal.next().map(|v| (self.times.next().unwrap(), v))
    }
}

#[derive(Debug, Clone)]
pub struct Filter<S, F> {
    signal: S,
    filter: F,
}

impl<S, F> Filter<S, F>
where
    S: Signal,
    F: filter::Filter<S::Sample>,
{
    pub(super) fn new<D>(signal: S, fd: D) -> Self
    where
        D: filter::FilterDesign<S::Sample, Filter=F, Output=F::Output>,
    {
        Filter {
            filter: fd.design_for(&signal),
            signal: signal,
        }
    }
}

impl<S, F> Signal for Filter<S, F>
where
    S: Signal,
    F: filter::Filter<S::Sample>,
{
    type Sample = F::Output;
    fn next(&mut self) -> Option<Self::Sample> {
        self.signal.next().map(|v| self.filter.apply(v))
    }
    fn rate(&self) -> f32 {
        self.signal.rate()
    }
}

#[derive(Debug, Clone)]
pub struct Iter<S> {
    signal: S,
}

impl<S> Iter<S> where S: Signal {
    pub(super) fn new(signal: S) -> Self {
        Iter { signal }
    }
}

impl<S> Iterator for Iter<S> where S: Signal {
    type Item = S::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        self.signal.next()
    }
}

impl<S> rodio::Source for Iter<S>
where
    S: Signal,
    S::Sample: rodio::Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        self.signal.rate().round() as u32
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct Map<S, F> {
    signal: S,
    f: F,
}

impl<S, F> Map<S, F> {
    pub(super) fn new(signal: S, f: F) -> Self {
        Map { signal, f }
    }
}

impl<S, F, A> Signal for Map<S, F>
where
    F: FnMut(S::Sample) -> A,
    S: Signal,
{
    type Sample = A;
    fn next(&mut self) -> Option<Self::Sample> {
        self.signal.next().map(&mut self.f)
    }
    fn rate(&self) -> f32 {
        self.signal.rate()
    }
}

#[derive(Clone, Debug)]
pub struct Skip<S> {
    signal: S,
    duration: usize,
}

impl<S> Skip<S> where S: Signal {
    pub(super) fn new(signal: S, duration: f32) -> Self {
        Skip {
            duration: (signal.rate() * duration).round() as usize,
            signal,
        }
    }
}

impl<S> Signal for Skip<S> where S: Signal {
    type Sample = S::Sample;
    fn next(&mut self) -> Option<Self::Sample> {
        while self.duration > 0 {
            self.duration -= 1;
            if let None = self.signal.next() {
                return None;
            }
        }
        self.signal.next()
    }
    fn rate(&self) -> f32 {
        self.signal.rate()
    }
}

#[derive(Debug, Clone)]
pub struct Stereo<S: Signal> {
    signal: S,
    sample: Option<S::Sample>,
}

impl<S> Stereo<S> where S: Signal {
    pub(super) fn new(signal: S) -> Self {
        Stereo { signal, sample: None }
    }
}

impl<S, A> Iterator for Stereo<S> where S: Signal<Sample=(A, A)>, A: Copy {
    type Item = A;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(s) = self.sample {
            self.sample = None;
            Some(s.1.clone())
        } else {
            self.sample = self.signal.next();
            self.sample.map(|v| v.0)
        }
    }
}

impl<S, A> rodio::Source for Stereo<S>
where
    S: Signal<Sample=(A, A)>,
    A: rodio::Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        2
    }
    fn sample_rate(&self) -> u32 {
        self.signal.rate().round() as u32
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct Take<S> {
    signal: S,
    duration: usize,
}

impl<S> Take<S> where S: Signal {
    pub(super) fn new(signal: S, duration: f32) -> Self {
        Take {
            duration: (signal.rate() * duration).round() as usize,
            signal,
        }
    }
}

impl<S> Signal for Take<S> where S: Signal {
    type Sample = S::Sample;
    fn next(&mut self) -> Option<Self::Sample> {
        if self.duration > 0 {
            self.duration -= 1;
            self.signal.next()
        } else {
            None
        }
    }
    fn rate(&self) -> f32 {
        self.signal.rate()
    }
}

#[derive(Debug)]
pub struct Window<S: Signal> {
    signal: S,
    buffer: std::rc::Rc<std::cell::RefCell<VecDeque<S::Sample>>>,
}

impl<S> Window<S> where S: Signal, S::Sample: num::Zero + Clone {
    pub(super) fn new(signal: S, duration: f32) -> Self {
        use num::Zero;
        let cap = (duration * signal.rate()).round() as usize;
        let buffer = std::iter::repeat(S::Sample::zero()).take(cap).collect();
        Window {
            signal,
            buffer: std::rc::Rc::new(std::cell::RefCell::new(buffer)),
        }
    }
}

impl<S> Signal for Window<S> where S: Signal {
    type Sample = std::rc::Rc<std::cell::RefCell<VecDeque<S::Sample>>>;
    fn next(&mut self) -> Option<Self::Sample> {
        if let Some(v) = self.signal.next() {
            let mut buf = self.buffer.borrow_mut();
            buf.pop_front();
            buf.push_back(v);
            Some(self.buffer.clone())
        } else {
            None
        }
    }
    fn rate(&self) -> f32 {
        self.signal.rate()
    }
}
