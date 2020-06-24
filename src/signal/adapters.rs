use super::Signal;
use super::times::Times;
use crate::filter;
use crate::resample;

use std::collections::VecDeque;

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
    pub(super) fn new<IF>(signal: S, fir: IF) -> Self
    where
        IF: filter::IntoFilter<S::Sample, Filter=F>,
    {
        Filter {
            filter: fir.into_filter(signal.rate()),
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

impl<S> Iterator for Enumerate<S> where S: Signal {
    type Item = (f32, S::Sample);
    fn next(&mut self) -> Option<Self::Item> {
        // unwrap is safe: times is infinite
        self.signal.next().map(|v| (self.times.next().unwrap(), v))
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
pub struct Resample<S: Signal> {
    signal: S,
    sr: resample::SampleRate<S::Sample>,
    rate: f32,
    ratio: f64,
    buffer: Vec<S::Sample>,
    buffer_resampled: Vec<S::Sample>,
    buffer_size: usize,
    buffer_next: usize,
    done: bool,
}

impl<S> Resample<S> where S: Signal, S::Sample: resample::Resample {
    pub(super) fn new(signal: S, typ: resample::ConverterType, rate: f32)
                      -> Self
    {
        let buffer_size = 4096;
        Resample {
            rate: rate,
            sr: resample::SampleRate::new(typ).unwrap(),
            ratio: rate as f64 / signal.rate() as f64,
            signal,
            buffer_size,
            buffer: Vec::with_capacity(buffer_size),
            buffer_resampled: Vec::with_capacity(buffer_size),
            buffer_next: buffer_size,
            done: false,
        }
    }
}

impl<S> Signal for Resample<S> where S: Signal, S::Sample: resample::Resample {
    type Sample = S::Sample;
    fn next(&mut self) -> Option<Self::Sample> {
        // early exit
        if self.done {
            return None
        }

        while self.buffer_next >= self.buffer_resampled.len() {
            // refill our buffer
            while self.buffer.len() < self.buffer_size {
                if let Some(v) = self.signal.next() {
                    self.buffer.push(v);
                } else {
                    break;
                }
            }

            // resample buffer
            let input_used = self.sr.process(
                self.ratio,
                &self.buffer,
                &mut self.buffer_resampled,
            ).unwrap();

            // if we had 0 input (end of stream) and 0 output, we are done
            if self.buffer.len() == 0 && self.buffer_resampled.len() == 0 {
                self.done = true;
                return None;
            }

            // remove used data
            self.buffer.splice(0..input_used, std::iter::empty());

            if self.buffer_resampled.len() == 0 {
                // libsamplerate gave us nothing this time, try again
                continue;
            }

            // reset our counter
            self.buffer_next = 0;
        }

        let v = self.buffer_resampled[self.buffer_next].clone();
        self.buffer_next += 1;
        Some(v)
    }
    fn rate(&self) -> f32 {
        self.rate
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
pub struct TeeBuffer<S, A> {
    backlog: VecDeque<A>,
    signal: S,
    owner: bool,
}

#[derive(Debug)]
pub struct Tee<S: Signal> {
    // we really, really want this to be Send, so
    // this could almost certainly be done smarter
    buffer: std::sync::Arc<std::sync::Mutex<TeeBuffer<S, S::Sample>>>,
    id: bool,
    rate: f32,
}

impl<S> Tee<S> where S: Signal {
    pub(super) fn new(signal: S) -> (Tee<S>, Tee<S>) {
        let buffer = TeeBuffer {
            backlog: VecDeque::new(),
            signal,
            owner: false,
        };
        let t1 = Tee {
            rate: buffer.signal.rate(),
            buffer: std::sync::Arc::new(std::sync::Mutex::new(buffer)),
            id: true,
        };
        let t2 = Tee {
            rate: t1.rate,
            buffer: t1.buffer.clone(),
            id: false,
        };
        (t1, t2)
    }
}

impl<S> Signal for Tee<S> where S: Signal, S::Sample: Clone {
    type Sample = S::Sample;
    fn next(&mut self) -> Option<Self::Sample> {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.owner == self.id {
            match buffer.backlog.pop_front() {
                None => (),
                some => return some,
            }
        }
        match buffer.signal.next() {
            None => None,
            Some(v) => {
                buffer.backlog.push_back(v.clone());
                buffer.owner = !self.id;
                Some(v)
            },
        }
    }
    fn rate(&self) -> f32 {
        self.rate
    }
}
