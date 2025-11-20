use malachite_base::num::basic::traits::{NegativeOne, One, Two, Zero};

use super::base::{Acci, OPitch, Step};
use super::interval::{IntervalDeg, IntervalQual, SimpleInterval};

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
        step: Step::C,
        tone: 0,
    };
    pub const D: Self = OPitch {
        step: Step::D,
        tone: 2,
    };
    pub const E: Self = OPitch {
        step: Step::E,
        tone: 4,
    };
    pub const F: Self = OPitch {
        step: Step::F,
        tone: 5,
    };
    pub const G: Self = OPitch {
        step: Step::G,
        tone: 7,
    };
    pub const A: Self = OPitch {
        step: Step::A,
        tone: 9,
    };
    pub const B: Self = OPitch {
        step: Step::B,
        tone: 11,
    };
}

impl Zero for OPitch {
    const ZERO: Self = OPitch {
        step: Step::C,
        tone: 0,
    };
}

impl Zero for Step {
    const ZERO: Self = Step::C;
}

impl Zero for IntervalDeg {
    const ZERO: Self = IntervalDeg::Unison;
}

impl Zero for SimpleInterval {
    const ZERO: Self = SimpleInterval {
        deg: IntervalDeg::Unison,
        qual: IntervalQual::Perfect,
    };
}
