use super::{Convolve, Fir, IntoFilter};

use nalgebra::{DMatrix, DVector};
use special_fun::FloatSpecial;

#[derive(Clone, Debug)]
pub enum Derivative {
    // type, derivative order, accuracy
    Center(usize, usize),
    Forward(usize, usize),
    Backward(usize, usize),
}

impl Derivative {
    pub fn derivative(&self) -> usize {
        match self {
            Derivative::Center(deriv, _) => *deriv,
            Derivative::Forward(deriv, _) => *deriv,
            Derivative::Backward(deriv, _) => *deriv,
        }
    }

    pub fn accuracy(&self) -> usize {
        match self {
            Derivative::Center(_, acc) => *acc,
            Derivative::Forward(_, acc) => *acc,
            Derivative::Backward(_, acc) => *acc,
        }
    }
    fn make_coef(self, rate: f32) -> Vec<f32> {
        let factor = rate.powi(self.derivative() as i32);
        let coef = match self {
            Derivative::Center(deriv, mut acc) => {
                // acc must be even, for central differences
                if acc % 2 != 0 {
                    acc += 1;
                }
                let size = 2 * ((deriv + 1) / 2) - 1 + acc;
                let half = size as isize / 2;
                self.make_coef_from_taps(-half, half, deriv)
            },
            Derivative::Forward(deriv, acc) => {
                let size = (deriv + acc) as isize;
                self.make_coef_from_taps(0, size - 1, deriv)
            },
            Derivative::Backward(deriv, acc) => {
                let size = (deriv + acc) as isize;
                self.make_coef_from_taps(-size + 1, 0, deriv)
            },
        };
        let mut vec: Vec<f32> = coef.into_iter().map(|c| c * factor).collect();
        vec.reverse();
        vec
    }

    fn make_coef_from_taps(&self, left: isize, right: isize, deriv: usize)
                           -> DVector<f32>
    {
        let n = (-left + right + 1) as usize;
        let mut matrix = DMatrix::<f32>::from_element(n, n, 0.0);
        for i in 0..n {
            for j in 0..n {
                matrix[(i, j)] = ((left + j as isize) as f32).powi(i as i32);
            }
        }

        let mut rhs = DVector::<f32>::from_element(n, 0.0);
        rhs[deriv] = (deriv as f32).factorial();

        matrix.lu().solve(&rhs).expect("finite difference failed")
    }
}

impl<A> IntoFilter<A> for Derivative where A: Convolve<f32 >{
    type Filter = Fir<f32, A>;
    fn into_filter(self, rate: f32) -> Self::Filter {
        Fir::new(self.make_coef(rate))
    }
}
