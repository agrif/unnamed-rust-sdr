use super::{Convolve, Filter, IntoFilter};

#[derive(Clone, Debug)]
pub struct Biquad<C, A> {
    // coefficients
    b0: C,
    b1: C,
    b2: C,
    na1: C,
    na2: C,

    // state
    x1: A,
    x2: A,
    y1: A,
    y2: A,
}

impl<C, A> Biquad<C, A>
where
    A: Convolve<C>,
    C: Clone + std::ops::Div<C, Output=C> + std::ops::Neg<Output=C>,
{
    pub fn new(a0: C, a1: C, a2: C, b0: C, b1: C, b2: C) -> Self {
        Biquad {
            b0: b0 / a0.clone(),
            b1: b1 / a0.clone(),
            b2: b2 / a0.clone(),
            na1: -a1 / a0.clone(),
            na2: -a2 / a0.clone(),

            x1: A::zero(),
            x2: A::zero(),
            y1: A::zero(),
            y2: A::zero(),
        }
    }
}

impl<C, A> Filter<A> for Biquad<C, A> where A: Convolve<C> {
    fn apply(&mut self, value: A) -> A {
        let mut out = A::zero();
        out.accumulate(&value, &self.b0);
        out.accumulate(&self.x1, &self.b1);
        out.accumulate(&self.x2, &self.b2);
        out.accumulate(&self.y1, &self.na1);
        out.accumulate(&self.y2, &self.na2);

        std::mem::swap(&mut self.x2, &mut self.x1);
        self.x1 = value;
        std::mem::swap(&mut self.y2, &mut self.y1);
        self.y1 = out.clone();
        out
    }
}

impl<C, A> IntoFilter<A> for Biquad<C, A> where A: Convolve<C> {
    type Filter = Biquad<C, A>;
    fn into_filter(self, _rate: f32) -> Self::Filter {
        Biquad {
            x1: A::zero(),
            x2: A::zero(),
            y1: A::zero(),
            y2: A::zero(),
            .. self
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Biquadratic {
    LowPass(f32, f32),
    HighPass(f32, f32),
    BandPass(f32, f32),
    Notch(f32, f32),

    Lr(f32),
}

impl<A> IntoFilter<A> for Biquadratic where A: Convolve<f32> {
    type Filter = Biquad<f32, A>;
    fn into_filter(self, rate: f32) -> Self::Filter {
        use Biquadratic::*;
        use std::f32::consts::PI;
        match self {
            LowPass(freq, q) => {
                let omega = 2.0 * PI * freq / rate;
                let cos = omega.cos();
                let alpha = omega.sin() / (2.0 * q);
                Biquad::new(
                    1.0 + alpha,
                    -2.0 * cos,
                    1.0 - alpha,
                    (1.0 - cos) / 2.0,
                    1.0 - cos,
                    (1.0 - cos) / 2.0,
                )
            },
            HighPass(freq, q) => {
                let omega = 2.0 * PI * freq / rate;
                let cos = omega.cos();
                let alpha = omega.sin() / (2.0 * q);
                Biquad::new(
                    1.0 + alpha,
                    -2.0 * cos,
                    1.0 - alpha,
                    (1.0 + cos) / 2.0,
                    -1.0 - cos,
                    (1.0 + cos) / 2.0,
                )
            },
            BandPass(freq, q) => {
                let omega = 2.0 * PI * freq / rate;
                let cos = omega.cos();
                let alpha = omega.sin() / (2.0 * q);
                Biquad::new(
                    1.0 + alpha,
                    -2.0 * cos,
                    1.0 - alpha,
                    alpha,
                    0.0,
                    -alpha,
                )
            },
            Notch(freq, q) => {
                let omega = 2.0 * PI * freq / rate;
                let cos = omega.cos();
                let alpha = omega.sin() / (2.0 * q);
                Biquad::new(
                    1.0 + alpha,
                    -2.0 * cos,
                    1.0 - alpha,
                    1.0,
                    -2.0 * cos,
                    1.0,
                )
            },
            Lr(decayrate) => {
                let decayn = decayrate / rate;
                Biquad::new(
                    1.0,
                    -(-decayn).exp(),
                    0.0,
                    decayn,
                    0.0,
                    0.0,
                )
            }
        }
    }
}
