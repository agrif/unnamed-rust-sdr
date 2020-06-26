use crate::Signal;
use crate::resample;

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
    pub(crate) fn new(signal: S, typ: resample::ConverterType, rate: f32)
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
