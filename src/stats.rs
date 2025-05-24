use parking_lot::RwLock;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

pub struct Stats {
    clicks: AtomicU64,
    session_start: RwLock<Instant>,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            clicks: AtomicU64::new(0),
            session_start: RwLock::new(Instant::now()),
        }
    }

    pub fn increment_clicks(&self) {
        self.clicks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_clicks(&self) -> u64 {
        self.clicks.load(Ordering::Relaxed)
    }

    pub fn get_cpm(&self) -> u64 {
        let elapsed = self.session_start.read().elapsed().as_secs();
        if elapsed == 0 {
            0
        } else {
            (self.get_clicks() * 60) / elapsed
        }
    }

    pub fn get_runtime(&self) -> Duration {
        self.session_start.read().elapsed()
    }

    pub fn reset(&self) {
        self.clicks.store(0, Ordering::Relaxed);
        *self.session_start.write() = Instant::now();
    }
}