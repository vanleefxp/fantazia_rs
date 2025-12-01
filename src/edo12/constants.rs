use malachite_base::num::basic::traits::{NegativeOne, One, Two, Zero};

use crate::edo12::IntervalDeg;

use super::base::{Acci, OPitch, Pitch, OStep, Step};
use super::interval::{SimpleIntervalDeg, IntervalQual, SimpleInterval, Interval};

impl Acci {
    pub const TRIPLE_FLAT: Self = Acci(-3);
    pub const DOUBLE_FLAT: Self = Acci(-2);
    pub const FLAT: Self = Acci(-1);
    pub const NATURAL: Self = Acci(0);
    pub const SHARP: Self = Acci(1);
    pub const DOUBLE_SHARP: Self = Acci(2);
    pub const TRIPLE_SHARP: Self = Acci(3);
}

impl Zero for Acci {
    const ZERO: Self = Self::NATURAL;
}

impl One for Acci {
    const ONE: Self = Self::SHARP;
}

impl Two for Acci {
    const TWO: Self = Self::DOUBLE_SHARP;
}

impl NegativeOne for Acci {
    const NEGATIVE_ONE: Self = Self::FLAT;
}

impl OPitch {
    pub const C: Self = OPitch {
        step: OStep::C,
        tone: 0,
    };
    pub const D: Self = OPitch {
        step: OStep::D,
        tone: 2,
    };
    pub const E: Self = OPitch {
        step: OStep::E,
        tone: 4,
    };
    pub const F: Self = OPitch {
        step: OStep::F,
        tone: 5,
    };
    pub const G: Self = OPitch {
        step: OStep::G,
        tone: 7,
    };
    pub const A: Self = OPitch {
        step: OStep::A,
        tone: 9,
    };
    pub const B: Self = OPitch {
        step: OStep::B,
        tone: 11,
    };
}

impl Zero for OPitch {
    const ZERO: Self = OPitch {
        step: OStep::C,
        tone: 0,
    };
}

impl Zero for Pitch {
    const ZERO: Self = Pitch {
        step: Step(0),
        tone: 0,
    };
}

impl Zero for OStep {
    const ZERO: Self = OStep::C;
}

impl Zero for SimpleIntervalDeg {
    const ZERO: Self = SimpleIntervalDeg::Unison;
}

impl Zero for IntervalDeg {
    const ZERO: Self = IntervalDeg(0);
}

impl Zero for SimpleInterval {
    const ZERO: Self = SimpleInterval {
        deg: SimpleIntervalDeg::Unison,
        qual: IntervalQual::Perfect,
    };
}

impl Zero for Interval {
    const ZERO: Self = Interval {
        deg: IntervalDeg::ZERO,
        qual: IntervalQual::Perfect,
    };
}

