use crate::Signal;

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
struct TeeDequeShared<A> {
    data: VecDeque<A>,
    available: Vec<usize>,
}

#[derive(Debug)]
struct TeeDeque<A> {
    shared: Arc<(Mutex<TeeDequeShared<A>>, Condvar)>,
    id: usize,
}

#[derive(Clone, Debug)]
struct TeeDequePush<A> {
    shared: Arc<(Mutex<TeeDequeShared<A>>, Condvar)>,
}

impl<A> TeeDeque<A> {
    fn new() -> Self {
        Self::with_capacity(0)
    }

    fn with_capacity(capacity: usize) -> Self {
        TeeDeque {
            shared: Arc::new((Mutex::new(
                TeeDequeShared {
                    data: VecDeque::with_capacity(capacity),
                    available: vec![0],
                }
            ), Condvar::new())),
            id: 0,
        }
    }

    fn pusher(&self) -> TeeDequePush<A> {
        TeeDequePush {
            shared: self.shared.clone(),
        }
    }

    fn try_pop<F>(&mut self, view: F) -> usize where F: FnOnce(Option<&A>) {
        let mut shared = self.shared.0.lock().unwrap();
        let avail = &mut shared.available[self.id];
        if *avail > 0 {
            *avail -= 1;
            let i = *avail;
            view(Some(&shared.data[i]));
            i
        } else {
            view(None);
            0
        }
    }

    fn pop<F>(&mut self, view: F) -> usize where F: FnOnce(&A) {
        let mut shared = self.shared.0.lock().unwrap();
        let mut avail = shared.available[self.id];
        while avail == 0 {
            shared = self.shared.1.wait(shared).unwrap();
            avail = shared.available[self.id];
        }
        avail -= 1;
        shared.available[self.id] = avail;
        view(&shared.data[avail]);
        avail
    }
}

impl<A> TeeDequePush<A> {
    // reduce. re-use. recycle.
    fn push<F>(&mut self, modify: F) where F: FnOnce(Option<A>) -> A {
        let mut shared = self.shared.0.lock().unwrap();
        let maxavail = shared.available.iter().max().unwrap_or(&0);
        let recycle = if *maxavail < shared.data.len() {
            Some(shared.data.pop_back().unwrap())
        } else {
            None
        };
        shared.data.push_front(modify(recycle));
        for avail in &mut shared.available {
            *avail += 1;
        }
        self.shared.1.notify_all();
    }
}

impl<A> Clone for TeeDeque<A> {
    fn clone(&self) -> Self {
        let mut shared = self.shared.0.lock().unwrap();
        let newid = shared.available.len();
        let newavail = shared.data.len();
        shared.available.push(newavail);
        TeeDeque {
            shared: self.shared.clone(),
            id: newid,
        }
    }
}

#[derive(Debug)]
pub struct Block<S: Signal> {
    signal: Arc<Mutex<S>>,
    rate: f32,
    data: TeeDeque<Vec<S::Sample>>,
    block_size: usize,
    current: Vec<S::Sample>,
    i: usize,
}

impl<S> Block<S> where S: Signal, S::Sample: Clone {
    pub(crate) fn new(signal: S, size: f32) -> Self {
        let block_size = (size * signal.rate()).ceil() as usize;
        Block {
            rate: signal.rate(),
            signal: Arc::new(Mutex::new(signal)),
            data: TeeDeque::new(),
            block_size,
            current: Vec::with_capacity(block_size),
            i: 0,
        }
    }
}

impl<S> Clone for Block<S> where S: Signal {
    fn clone(&self) -> Self {
        Block {
            signal: self.signal.clone(),
            rate: self.rate,
            data: self.data.clone(),
            block_size: self.block_size,
            current: Vec::with_capacity(self.block_size),
            i: 0,
        }
    }
}

impl<S> Signal for Block<S>
where
    S: Signal + Send + 'static,
    S::Sample: Clone + Send + 'static,
{
    type Sample = S::Sample;
    fn next(&mut self) -> Option<S::Sample> {
        if self.i < self.current.len() {
            let r = self.current[self.i].clone();
            self.i += 1;
            Some(r)
        } else {
            self.current.clear();
            self.i = 0;
            let mut needs_extra = true;
            let current = &mut self.current;
            let avail = self.data.try_pop(|mn| {
                if let Some(next) = mn {
                    current.extend_from_slice(next);
                    needs_extra = false;
                }
            });

            let target = 1;
            if avail < target {
                let mut push = self.data.pusher();
                let block_size = self.block_size;
                let signalmutex = self.signal.clone();
                let mut blockjobs = target - avail;
                if needs_extra {
                    blockjobs += 1;
                }
                rayon::spawn_fifo(move || {
                    for _ in 0..blockjobs {
                        push.push(|r| {
                            let mut v = r.unwrap_or_else(
                                || Vec::with_capacity(block_size));
                            v.clear();
                            let mut signal = signalmutex.lock().unwrap();
                            for _ in 0..block_size {
                                if let Some(val) = signal.next() {
                                    v.push(val);
                                }
                            }
                            v
                        })
                    }
                });
                if needs_extra {
                    self.data.pop(|next| {
                        current.extend_from_slice(next);
                    });
                }
            }

            if self.i < self.current.len() {
                Some(self.current[self.i].clone())
            } else {
                None
            }
        }
    }
    fn rate(&self) -> f32 {
        self.rate
    }
}
