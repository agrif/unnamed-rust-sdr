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

pub fn from_iter<I>(rate: f32, iter: I) -> FromIter<I>
where
    I: Iterator,
{
    FromIter::new(rate, iter)
}

#[derive(Debug, Clone)]
pub struct FromFunc<F> {
    times: Times,
    func: F,
}

impl<F> FromFunc<F> {
    pub fn new(rate: f32, func: F) -> Self {
        FromFunc {
            times: Times::new(rate),
            func,
        }
    }
}

impl<F, A> Signal for FromFunc<F> where F: FnMut(f32) -> A {
    type Sample = A;
    fn next(&mut self) -> Option<Self::Sample> {
        self.times.next().map(&mut self.func)
    }
    fn rate(&self) -> f32 {
        self.times.rate()
    }
}

pub fn from_func<F, A>(rate: f32, func: F) -> FromFunc<F>
where
    F: FnMut(f32) -> A,
{
    FromFunc::new(rate, func)
}

#[derive(Debug, Clone)]
pub struct Constant<A> {
    rate: f32,
    value: A,
}

impl<A> Constant<A> {
    pub fn new(rate: f32, value: A) -> Self {
        Constant {
            rate,
            value,
        }
    }
}

impl<A> Signal for Constant<A> where A: Clone {
    type Sample = A;
    fn next(&mut self) -> Option<Self::Sample> {
        Some(self.value.clone())
    }
    fn rate(&self) -> f32 {
        self.rate
    }
}

pub fn constant<A>(rate: f32, value: A) -> Constant<A>
where
    A: Clone,
{
    Constant::new(rate, value)
}

pub fn one<A>(rate: f32) -> Constant<A>
where
    A: num::One + Clone
{
    constant(rate, A::one())
}

pub fn zero<A>(rate: f32) -> Constant<A>
where
    A: num::Zero + Clone
{
    constant(rate, A::zero())
}

#[derive(Debug, Clone)]
pub struct FreqSweep {
    rate: f32,
    dt: f32,
    freq: f32,
    dfdt: f32,
    // from 0 to 1, just so we can keep it bounded
    nphase: f32,

    // sweep start, end times
    fstart: usize,
    fend: usize,

    // length of whole signal, or infinite
    length: Option<usize>,
}

impl FreqSweep {
    pub fn new(rate: f32, freq: f32, dfdt: f32, phase: f32,
               fstart: f32, fend: f32, length: Option<f32>) -> Self {
        FreqSweep {
            rate,
            dt: 1.0 / rate,
            freq,
            dfdt,
            nphase: phase / (2.0 * std::f32::consts::PI),
            fstart: (fstart * rate).round() as usize,
            fend: (fend * rate).round() as usize,
            length: length.map(|v| (v * rate).round() as usize),
        }
    }
}

impl Signal for FreqSweep {
    type Sample = (f32, Complex<f32>);
    fn next(&mut self) -> Option<Self::Sample> {
        if let Some(ref mut length) = self.length {
            if *length == 0 {
                return None;
            }
            *length -= 1;
        }

        let mut dfdt = self.dfdt;
        if self.fstart > 0 {
            self.fstart -= 1;
            dfdt = 0.0;
        }
        if self.fend > 0 {
            self.fend -= 1;
        } else {
            dfdt = 0.0;
        }

        self.freq += self.dt * dfdt;
        self.nphase += self.dt * self.freq;
        self.nphase = self.nphase.fract();
        let phase = 2.0 * std::f32::consts::PI * self.nphase;
        Some((self.freq, Complex::from_polar(&1.0, &phase)))
    }
    fn rate(&self) -> f32 {
        self.rate
    }
}

pub fn freq_sweep(rate: f32, df: f32, warmup: bool, range: std::ops::Range<f32>)
                  -> FreqSweep
{
    // df is frequency resolution, not df/dt
    // (easy proof: df has units frequency, df/dt has frequency^2)
    let mut dfdt = df.powi(2);
    if range.start > range.end {
        dfdt = -dfdt;
    }
    let endt = (range.end - range.start) / dfdt;
    let warmupt = if warmup { 1.0 / df } else { 0.0 };
    FreqSweep::new(rate, range.start, dfdt, 0.0,
                   warmupt, warmupt + endt, Some(warmupt + endt))
}

#[derive(Clone, Debug)]
pub struct Freq {
    sweep: FreqSweep,
}

impl Freq {
    pub fn new(rate: f32, freq: f32, phase: f32) -> Self {
        Freq {
            sweep: FreqSweep::new(rate, freq, 0.0, phase, 0.0, 0.0, None)
        }
    }
}

impl Signal for Freq {
    type Sample = Complex<f32>;
    fn next(&mut self) -> Option<Self::Sample> {
        self.sweep.next().map(|t| t.1)
    }
    fn rate(&self) -> f32 {
        self.sweep.rate()
    }
}

pub fn freq(rate: f32, freq: f32, phase: f32) -> Freq {
    Freq::new(rate, freq, phase)
}

pub struct Impulse<A> {
    rate: f32,
    first: bool,
    _marker: std::marker::PhantomData<A>,
}

impl<A> Impulse<A> {
    pub fn new(rate: f32) -> Self {
        Impulse {
            rate,
            first: true,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<A> Signal for Impulse<A> where A: num::Zero + num::One {
    type Sample = A;
    fn next(&mut self) -> Option<Self::Sample> {
        let first = self.first;
        self.first = false;
        if first {
            Some(A::one())
        } else {
            Some(A::zero())
        }
    }
    fn rate(&self) -> f32 {
        self.rate
    }
}

pub fn impulse<A>(rate: f32) -> Impulse<A> where A: num::Zero + num::One {
    Impulse::new(rate)
}
