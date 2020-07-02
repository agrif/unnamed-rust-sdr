use super::{Filter, FilterDesign};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity;

impl<A> Filter<A> for Identity {
    type Output = A;
    fn apply(&mut self, value: A) -> Self::Output {
        value
    }
}

impl<A> FilterDesign<A> for Identity {
    type Output = A;
    type Filter = Identity;
    fn design(self, _rate: f32) -> Self::Filter {
        Identity
    }
}

#[derive(Clone, Debug)]
pub struct MonitorD<F>(pub f32, pub F);

#[derive(Clone, Debug)]
pub struct Monitor<F> {
    every: usize,
    i: usize,
    func: F,
}

impl<F, A> Filter<A> for Monitor<F> where F: FnMut(&A) -> () {
    type Output = A;
    fn apply(&mut self, value: A) -> Self::Output {
        self.i += 1;
        if self.i >= self.every {
            self.i = 0;
            (self.func)(&value);
        }
        value
    }
}

impl<F, A> FilterDesign<A> for MonitorD<F> where F: FnMut(&A) -> () {
    type Output = A;
    type Filter = Monitor<F>;
    fn design(self, rate: f32) -> Self::Filter {
        Monitor {
            every: (rate / self.0).round() as usize,
            i: 0,
            func: self.1,
        }
    }
}
