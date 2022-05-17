use std::time::{Duration, Instant};

pub trait TimeSource {
    fn elapsed(&self) -> Duration;
}

#[derive(Clone)]
pub struct SeekableTimeSource {
    base: Instant,
    offset: Duration,
    paused: bool,
}

impl TimeSource for SeekableTimeSource {
    fn elapsed(&self) -> Duration {
        if self.paused {
            self.offset
        } else {
            self.base.elapsed() + self.offset
        }
    }
}

impl SeekableTimeSource {
    pub fn now() -> SeekableTimeSource {
        SeekableTimeSource {
            base: Instant::now(),
            offset: Duration::ZERO,
            paused: false,
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.offset = self.elapsed();
        self.base = Instant::now();
        self.paused = paused;
    }

    pub fn seek(&mut self, pos: Duration) {
        self.offset = pos;
        self.base = Instant::now();
    }
}
