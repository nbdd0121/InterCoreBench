use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Semaphore(AtomicUsize);

impl Semaphore {
    pub fn new(value: usize) -> Self {
        Self(AtomicUsize::new(value))
    }

    pub fn up_n(&self, n: usize) {
        self.0.fetch_add(n, Ordering::Relaxed);
    }

    pub fn up(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    pub fn down(&self) {
        let mut prev = self.0.load(Ordering::Relaxed);
        loop {
            if prev == 0 {
                prev = self.0.load(Ordering::Relaxed);
                continue;
            }
            match self
                .0
                .compare_exchange_weak(prev, prev - 1, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => return,
                Err(next_prev) => prev = next_prev,
            }
        }
    }
}
