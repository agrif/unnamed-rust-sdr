use super::Signal;
use super::times::Times;
use crate::fir::{Fir, IntoFir, Convolve};
use crate::resample;

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
pub struct Filter<S, C> where S: Signal {
    signal: S,
    fir: Fir<C, S::Sample>,
}

impl<S, C> Filter<S, C> where S: Signal, S::Sample: Convolve<C> {
    pub(super) fn new<F>(signal: S, fir: F) -> Self where F: IntoFir<C> {
        Filter {
            fir: fir.into_fir(signal.rate()),
            signal: signal,
        }
    }
}

impl<S, C> Signal for Filter<S, C>
where
    S: Signal,
    S::Sample: Convolve<C>,
{
    type Sample = S::Sample;
    fn next(&mut self) -> Option<Self::Sample> {
        while let Some(v) = self.signal.next() {
            if let Some(r) = self.fir.apply(v) {
                return Some(r)
            }
        }
        None
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
        self.signal.next().map(|x| (self.f)(x))
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
