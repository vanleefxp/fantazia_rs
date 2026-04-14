use std::cmp::Ordering;

use crate::rhythm::BinaryDuration;

impl Ord for BinaryDuration {
    fn cmp(&self, other: &Self) -> Ordering {
        self.kind
            .cmp(&other.kind)
            .reverse()
            .then_with(|| self.dots.cmp(&other.dots))
    }
}

impl PartialOrd for BinaryDuration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
