use super::Signal;
use super::times::Times;
use crate::fir::{Fir, IntoFir, Convolve};

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
    state: *mut libsamplerate::SRC_STATE,
    rate: f32,
    ratio: f32,
    buffer: Vec<S::Sample>,
    buffer_resampled: Vec<S::Sample>,
    buffer_size: usize,
    buffer_next: usize,
}

// the way we use state, this is safe
unsafe impl<S> Send for Resample<S> where S: Signal {}

pub trait Channels: Sized + Copy {
    fn channels() -> usize;
}

impl Channels for f32 {
    // in addition: this guarantees that casting from *Self to *f32 is fine
    fn channels() -> usize {
        1
    }
}

impl<S> Resample<S> where S: Signal, S::Sample: Channels {
    pub(super) fn new(signal: S, rate: f32) -> Self {
        let buffer_size = 4096;
        Resample {
            rate: rate,
            state: unsafe {
                let state = libsamplerate::samplerate::src_new(
                    libsamplerate::src_sinc::SRC_SINC_BEST_QUALITY as i32,
                    S::Sample::channels() as i32,
                    std::ptr::null_mut(),
                );
                if state.is_null() {
                    panic!("could not initialize libsamplerate");
                }
                state
            },
            ratio: rate / signal.rate(),
            signal,
            buffer_size,
            buffer: Vec::with_capacity(buffer_size),
            buffer_resampled: Vec::with_capacity(buffer_size),
            buffer_next: buffer_size,
        }
    }
}

impl<S> Drop for Resample<S> where S: Signal {
    fn drop(&mut self) {
        if !self.state.is_null() {
            unsafe {
                self.state = libsamplerate::samplerate::src_delete(self.state);
            }
        }
    }
}

impl<S> Signal for Resample<S> where S: Signal, S::Sample: Channels {
    type Sample = S::Sample;
    fn next(&mut self) -> Option<Self::Sample> {
        while self.buffer_next >= self.buffer_resampled.len() {
            // refill our buffer
            while self.buffer.len() < self.buffer_size {
                if let Some(v) = self.signal.next() {
                    self.buffer.push(v);
                } else {
                    break;
                }
            }

            if self.buffer.len() == 0 {
                // we tried -- nothing left
                return None;
            }

            // resample buffer
            let mut src = libsamplerate::samplerate::SRC_DATA {
                data_in: self.buffer.as_mut_ptr() as *mut f32,
                data_out: self.buffer_resampled.as_mut_ptr() as *mut f32,
                input_frames: self.buffer.len() as i32,
                output_frames: self.buffer_resampled.capacity() as i32,
                input_frames_used: 0,
                output_frames_gen: 0,
                end_of_input: 0, // FIXME
                src_ratio: self.ratio as f64,
            };
            unsafe {
                if libsamplerate::samplerate::src_process(self.state, &mut src) > 0 {
                    panic!("libsamplerate failed");
                }
                self.buffer_resampled.set_len(src.output_frames_gen as usize);
            }
            self.buffer.splice(0..src.input_frames_used as usize, std::iter::empty());

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
