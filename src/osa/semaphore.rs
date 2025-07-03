use core::sync::atomic::{AtomicIsize, Ordering};

pub struct Semaphore {
    count: AtomicIsize,
}

impl Semaphore {
    pub fn new(count: isize) -> Self {
        Self { count: AtomicIsize::new(count) }
    }

    pub fn down(&self) {
        loop {
            let current = self.count.load(Ordering::Relaxed);
            if current > 0 {
                if self.count.compare_exchange(
                    current, current - 1, Ordering::Acquire, Ordering::Relaxed
                ).is_ok() {
                    break;
                }
            }
            core::hint::spin_loop();
        }
    }

    pub fn up(&self) {
        self.count.fetch_add(1, Ordering::Release);
    }
}