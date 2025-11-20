use num_enum::{IntoPrimitive, TryFromPrimitive};

use super::base::{OPitch, Step};

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub enum IntervalQual {
    Diminished(u8),
    Minor,
    Perfect,
    Major,
    Augmented(u8),
}

#[repr(u8)]
#[derive(IntoPrimitive, TryFromPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum IntervalDeg {
    Unison = 0,
    Second = 1,
    Third = 2,
    Fourth = 3,
    Fifth = 4,
    Sixth = 5,
    Seventh = 6,
}

impl From<Step> for IntervalDeg {
    fn from(value: Step) -> Self {
        IntervalDeg::try_from(u8::try_from(value).unwrap()).unwrap()
    }
}

impl From<IntervalDeg> for Step {
    fn from(value: IntervalDeg) -> Self {
        Step::try_from(u8::try_from(value).unwrap()).unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct SimpleInterval {
    pub(crate) deg: IntervalDeg,
    pub(crate) qual: IntervalQual,
}

impl SimpleInterval {
    pub fn deg(&self) -> IntervalDeg {
        self.deg
    }
    pub fn qual(&self) -> IntervalQual {
        self.qual
    }
}

impl OPitch {
    pub fn qual(&self) -> IntervalQual {
        use IntervalQual::*;
        use Step::*;
        let tone_diff = self.tone - self.step.diatonic_tone();
        match self.step {
            C | F | G => match tone_diff {
                0 => Perfect,
                n if n > 0 => Augmented(n as u8),
                n => Diminished((-n) as u8),
            },
            _ => match tone_diff {
                0 => Major,
                -1 => Minor,
                n if n > 0 => Augmented(n as u8),
                n => Diminished((-n - 1) as u8),
            },
        }
    }
}

impl From<OPitch> for SimpleInterval {
    fn from(value: OPitch) -> Self {
        let qual = value.qual();
        let deg = IntervalDeg::from(value.step);
        SimpleInterval { deg, qual }
    }
}

impl From<SimpleInterval> for OPitch {
    fn from(value: SimpleInterval) -> OPitch {
        use IntervalQual::*;
        use Step::*;
        let step: Step = value.deg.into();
        let mut tone = step.diatonic_tone();
        match step {
            C | F | G => match value.qual {
                Perfect => {}
                Augmented(n) => tone += n as i8,
                Diminished(n) => tone -= n as i8,
                _ => unreachable!(),
            },
            _ => match value.qual {
                Major => {}
                Minor => tone -= 1,
                Augmented(n) => tone += n as i8,
                Diminished(n) => tone -= n as i8 + 1,
                _ => unreachable!(),
            },
        }
        OPitch { step, tone }
    }
}
