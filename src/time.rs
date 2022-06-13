use std::{
    ops::{Add, Sub},
    time::Duration,
};

const NANOS_PER_FRAME: u64 = 1_000_000_000 / 60u64;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Time {
    frame: u32,
}

impl Time {
    pub(crate) fn from_frame(frame: u32) -> Time {
        Time { frame }
    }
}

impl Add for Time {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Time {
            frame: self.frame + rhs.frame,
        }
    }
}

impl Sub for Time {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Time {
            frame: self.frame - rhs.frame,
        }
    }
}

impl From<Duration> for Time {
    fn from(duration: Duration) -> Self {
        Time {
            frame: (duration.as_nanos() as u64 / NANOS_PER_FRAME) as u32,
        }
    }
}

impl From<Time> for Duration {
    fn from(time: Time) -> Self {
        Duration::from_secs_f64(time.frame as f64 * 1f64 / 60f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_from_duration() {
        let time: Time = Duration::from_secs(1).into();
        assert_eq!(60, time.frame);

        let time: Time = Duration::from_secs(5).into();
        assert_eq!(300, time.frame);

        let time: Time = Duration::from_secs_f64(1f64 / 60f64).into();
        assert_eq!(1, time.frame);

        let time: Time = Duration::from_secs_f64(8f64 / 60f64).into();
        assert_eq!(8, time.frame);

        let time: Time = Duration::from_secs_f64(9f64 / 60f64).into();
        assert_eq!(9, time.frame);
    }

    #[test]
    fn duration_from_time() {
        let duration: Duration = Time { frame: 60 }.into();
        assert_eq!(Duration::from_secs(1), duration);

        let duration: Duration = Time { frame: 300 }.into();
        assert_eq!(Duration::from_secs(5), duration);

        let duration: Duration = Time { frame: 1 }.into();
        assert_eq!(Duration::from_secs_f64(1f64 / 60f64), duration);

        let duration: Duration = Time { frame: 8 }.into();
        assert_eq!(Duration::from_secs_f64(8f64 / 60f64), duration);

        let duration: Duration = Time { frame: 9 }.into();
        assert_eq!(Duration::from_secs_f64(9f64 / 60f64), duration);
    }
}
