use malachite_base::num::comparison::traits::EqAbs;

use crate::edo12::{Interval, IntervalDeg, Pitch, Step};

macro_rules! derive_eq_abs {
    ($($t:ty),*$(,)?) => {
        $(
            impl EqAbs for $t {
                fn eq_abs(&self, other: &$t) -> bool {
                    self.0.eq_abs(&other.0)
                }
            }
        )*
    };
}

derive_eq_abs!(Step, IntervalDeg);

impl EqAbs for Pitch {
    fn eq_abs(&self, other: &Self) -> bool {
        self.step.eq_abs(&other.step) && self.tone.eq_abs(&other.tone)
    }
}

impl EqAbs for Interval {
    fn eq_abs(&self, other: &Self) -> bool {
        Pitch::from(*self).eq_abs(&Pitch::from(*other))
    }
}
