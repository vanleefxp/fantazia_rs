use super::super::interval::{IntervalQual, OInterval, Interval};
use std::cmp::Ordering;

macro_rules! derive_partial_ord_from_ord {
    ($($t:ty),*$(,)?) => {
        $(impl PartialOrd for $t {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        })*
    };
}

impl PartialOrd for IntervalQual {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::*;
        use IntervalQual::*;
        match (self, other) {
            (Augmented(a), Augmented(b)) | (Diminished(b), Diminished(a)) => a.partial_cmp(b),
            (a, b) if a == b => Some(Equal),
            (Augmented(_), _) | (_, Diminished(_)) => Some(Greater),
            (_, Augmented(_)) | (Diminished(_), _) => Some(Less),
            (Major, Minor) => Some(Greater),
            (Minor, Major) => Some(Less),
            _ => None,
        }
    }
}

impl Ord for OInterval {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deg.cmp(&other.deg).then_with(|| self.qual.partial_cmp(&other.qual).unwrap())
    }
}

impl Ord for Interval {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deg.cmp(&other.deg).then_with(|| self.qual.partial_cmp(&other.qual).unwrap())
    }
}

derive_partial_ord_from_ord!(OInterval, Interval);
