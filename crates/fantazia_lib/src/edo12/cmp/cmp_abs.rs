use std::cmp::Ordering;

use malachite_base::num::comparison::traits::{OrdAbs, PartialOrdAbs};

use crate::edo12::{Interval, IntervalDeg, IntervalQual, Pitch, Step};

macro_rules! derive_cmp_abs {
    ($($t:ty),*$(,)?) => {
        $(impl PartialOrdAbs for $t {
            fn partial_cmp_abs(&self, other: &Self) -> Option<Ordering> {
                self.0.partial_cmp_abs(&other.0)
            }
        }

        impl OrdAbs for $t {
            fn cmp_abs(&self, other: &Self) -> Ordering {
                self.0.cmp_abs(&other.0)
            }
        })*
    };
}

macro_rules! derive_partial_ord_abs_from_ord_abs {
    ($($t:ty),*$(,)?) => {
        $(
            impl PartialOrdAbs for $t {
                fn partial_cmp_abs(&self, other: &Self) -> Option<Ordering> {
                    Some(self.cmp_abs(other))
                }
            }
        )*
    };
}

derive_cmp_abs!(Step, IntervalDeg);

impl OrdAbs for Pitch {
    fn cmp_abs(&self, other: &Self) -> Ordering {
        self.step
            .cmp_abs(&other.step)
            .then_with(|| self.tone.cmp(&other.tone))
    }
}

impl OrdAbs for Interval {
    fn cmp_abs(&self, other: &Self) -> Ordering {
        Pitch::from(*self).cmp_abs(&Pitch::from(*other))
    }
}

impl PartialOrdAbs for IntervalQual {
    fn partial_cmp_abs(&self, other: &Self) -> Option<Ordering> {
        use IntervalQual::*;
        use Ordering::*;
        match (self, other) {
            (Augmented(n1) | Diminished(n1), Augmented(n2) | Diminished(n2)) => n1.partial_cmp(n2),
            (Perfect, Perfect) | (Major | Minor, Major | Minor) => Some(Equal),
            (Augmented(_) | Diminished(_), _) => Some(Greater),
            (_, Augmented(_) | Diminished(_)) => Some(Less),
            _ => None,
        }
    }
}

derive_partial_ord_abs_from_ord_abs!(Pitch, Interval);
