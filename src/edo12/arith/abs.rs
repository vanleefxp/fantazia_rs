use malachite_base::num::arithmetic::traits::{Abs, AbsAssign};

use super::super::{Interval, IntervalDeg, Pitch, Step};

impl Abs for Pitch {
    type Output = Self;

    fn abs(self) -> Self::Output {
        if self.step.0 > 0 {
            self
        } else if self.step.0 < 0 {
            Pitch {
                step: Step(-self.step.0),
                tone: -self.tone,
            }
        } else {
            Pitch {
                step: Step(0),
                tone: self.tone.abs(),
            }
        }
    }
}

impl Abs for Interval {
    type Output = Self;
    fn abs(self) -> Self::Output {
        Pitch::from(self).abs().into()
    }
}

impl Abs for Step {
    type Output = Self;
    fn abs(self) -> Self::Output {
        Step(self.0.abs())
    }
}

impl Abs for IntervalDeg {
    type Output = Self;
    fn abs(self) -> Self::Output {
        IntervalDeg(self.0.abs())
    }
}

impl AbsAssign for Pitch {
    fn abs_assign(&mut self) {
        *self = self.abs();
    }
}
