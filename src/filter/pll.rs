use super::{Filter, IntoFilter};

#[derive(Clone, Debug)]
pub struct PllDesign<Loop, Output, Lock> {
    reference: f32,
    gain: f32,
    loopfilter: Loop,
    outputfilter: Output,
    lockfilter: Lock,
}

#[derive(Clone, Debug)]
pub struct Pll<Loop, Output, Lock> {
    rate: f32,
    reference: f32,
    gain: f32,
    loopfilter: Loop,
    outputfilter: Output,
    lockfilter: Lock,

    nphase: f32,
    value: num::Complex<f32>,
}

impl<Loop, Output, Lock> PllDesign<Loop, Output, Lock> {
    pub fn new(reference: f32, gain: f32,
               loopfilter: Loop, outputfilter: Output, lockfilter: Lock)
               -> Self {
        PllDesign {
            reference,
            gain,
            loopfilter,
            outputfilter,
            lockfilter,
        }
    }
}

impl<Loop, Output, Lock> IntoFilter<num::Complex<f32>>
    for PllDesign<Loop, Output, Lock>
where
    Loop: IntoFilter<num::Complex<f32>>,
    Loop::Filter: Filter<num::Complex<f32>, Output=num::Complex<f32>>,
    Output: IntoFilter<f32>,
    Output::Filter: Filter<f32, Output=f32>,
    Lock: IntoFilter<f32>,
    Lock::Filter: Filter<f32, Output=f32>,
{
    type Filter = Pll<Loop::Filter, Output::Filter, Lock::Filter>;
    fn into_filter(self, rate: f32) -> Self::Filter {
        Pll {
            rate,
            reference: self.reference / rate,
            gain: self.gain, // FIXME this has something to do..
            loopfilter: self.loopfilter.into_filter(rate),
            outputfilter: self.outputfilter.into_filter(rate),
            lockfilter: self.lockfilter.into_filter(rate),

            nphase: 0.0,
            value: num::Complex::new(0.0, 0.0),
        }
    }
}

impl<Loop, Output, Lock> Filter<num::Complex<f32>> for Pll<Loop, Output, Lock>
where
    Loop: Filter<num::Complex<f32>, Output=num::Complex<f32>>,
    Output: Filter<f32, Output=f32>,
    Lock: Filter<f32, Output=f32>,
{
    type Output = Option<f32>;
    fn apply(&mut self, value: num::Complex<f32>) -> Self::Output {
        let c = value * self.value.conj();
        let phasedif = self.loopfilter.apply(c).arg() * self.gain;
        self.nphase += self.reference + phasedif;
        self.nphase = self.nphase.fract();
        let phase = 2.0 * std::f32::consts::PI * self.nphase;
        self.value = num::Complex::from_polar(&1.0, &phase);

        let locked = self.lockfilter.apply(c.re);
        let output = self.outputfilter.apply(phasedif * self.rate);
        if locked > 0.01 {
            Some(output)
        } else {
            None
        }
    }
}
