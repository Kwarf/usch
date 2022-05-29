use std::ops::{Add, Sub};

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Time {
    frame: u32,
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
