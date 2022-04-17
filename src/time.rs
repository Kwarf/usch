use std::time::{Duration, Instant};

pub trait TimeSource {
    fn elapsed(&self) -> Duration;
}

pub struct SeekableTimeSource {
    base: Instant,
    offset: Duration,
}

impl TimeSource for SeekableTimeSource {
    fn elapsed(&self) -> Duration {
        self.base.elapsed() + self.offset
    }
}

impl SeekableTimeSource {
    pub fn now() -> SeekableTimeSource {
        SeekableTimeSource {
            base: Instant::now(),
            offset: Duration::ZERO,
        }
    }

    pub fn seek(&mut self, pos: Duration) {
        self.base = Instant::now();
        self.offset = pos;
    }
}
