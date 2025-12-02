use derive_more as dm;
use malachite_base::num::arithmetic::traits::{DivMod as _, Mod as _};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use super::base::{Acci, OPitch, OStep, Pitch, Step};
use crate::{impl_from_mod, traits::FromMod};

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
pub enum SimpleIntervalDeg {
    Unison = 0,
    Second = 1,
    Third = 2,
    Fourth = 3,
    Fifth = 4,
    Sixth = 5,
    Seventh = 6,
}

impl_from_mod!(SimpleIntervalDeg, 7, u8; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

impl From<OStep> for SimpleIntervalDeg {
    fn from(value: OStep) -> Self {
        SimpleIntervalDeg::try_from(u8::try_from(value).unwrap()).unwrap()
    }
}

impl From<Step> for SimpleIntervalDeg {
    fn from(value: Step) -> Self {
        IntervalDeg::from(value).into()
    }
}

impl From<IntervalDeg> for SimpleIntervalDeg {
    fn from(value: IntervalDeg) -> Self {
        Self::from_mod(value.0)
    }
}

#[derive(
    dm::From,
    dm::Into,
    dm::Add,
    dm::Sub,
    dm::AddAssign,
    dm::SubAssign,
    dm::Neg,
    Clone,
    Copy,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Debug,
)]
pub struct IntervalDeg(pub(crate) i8);

impl IntervalDeg {
    pub fn octave(&self) -> i8 {
        self.0.div_mod(7).0
    }

    pub fn into_odeg_and_octave(&self) -> (SimpleIntervalDeg, i8) {
        let (octave, odeg) = self.0.div_mod(7);
        let odeg = odeg as u8;
        (SimpleIntervalDeg::try_from(odeg).unwrap(), octave)
    }
}

impl From<Step> for IntervalDeg {
    fn from(value: Step) -> Self {
        IntervalDeg(value.0)
    }
}

impl From<OStep> for IntervalDeg {
    fn from(value: OStep) -> Self {
        SimpleIntervalDeg::from(value).into()
    }
}

impl From<SimpleIntervalDeg> for IntervalDeg {
    fn from(value: SimpleIntervalDeg) -> Self {
        IntervalDeg(value as i8)
    }
}

pub trait Qual {
    fn qual(&self) -> IntervalQual;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct SimpleInterval {
    pub(crate) deg: SimpleIntervalDeg,
    pub(crate) qual: IntervalQual,
}

impl SimpleInterval {
    pub fn deg(&self) -> SimpleIntervalDeg {
        self.deg
    }
}

impl Qual for SimpleInterval {
    fn qual(&self) -> IntervalQual {
        self.qual
    }
}

impl From<Interval> for SimpleInterval {
    fn from(value: Interval) -> Self {
        SimpleInterval {
            deg: value.deg.into(),
            qual: value.qual,
        }
    }
}

impl From<OPitch> for SimpleInterval {
    fn from(value: OPitch) -> Self {
        let qual = value.qual();
        let deg = SimpleIntervalDeg::from(value.step);
        SimpleInterval { deg, qual }
    }
}

impl From<Pitch> for SimpleInterval {
    fn from(value: Pitch) -> Self {
        OPitch::from(value).into()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Interval {
    pub(crate) deg: IntervalDeg,
    pub(crate) qual: IntervalQual,
}

impl Interval {
    pub fn deg(&self) -> IntervalDeg {
        self.deg
    }

    pub fn odeg(&self) -> SimpleIntervalDeg {
        self.deg.into()
    }

    pub fn octave(&self) -> i8 {
        self.deg.octave()
    }
}

impl From<SimpleInterval> for Interval {
    fn from(value: SimpleInterval) -> Self {
        Interval {
            deg: value.deg.into(),
            qual: value.qual,
        }
    }
}

impl From<Pitch> for Interval {
    fn from(value: Pitch) -> Self {
        let qual = value.qual();
        let deg = IntervalDeg::from(value.step);
        Interval { deg, qual }
    }
}

impl From<OPitch> for Interval {
    fn from(value: OPitch) -> Self {
        Pitch::from(value).into()
    }
}

impl Qual for Interval {
    fn qual(&self) -> IntervalQual {
        self.qual
    }
}

impl Qual for OPitch {
    fn qual(&self) -> IntervalQual {
        use IntervalQual::*;
        use OStep::*;
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

impl Qual for Pitch {
    fn qual(&self) -> IntervalQual {
        OPitch::from(*self).qual()
    }
}

pub trait AcciByQual {
    fn acci_by_qual(&self, qual: IntervalQual) -> Option<Acci>;
}

impl AcciByQual for OStep {
    fn acci_by_qual(&self, qual: IntervalQual) -> Option<Acci> {
        use IntervalQual::*;
        use OStep::*;
        match self {
            C | F | G => match qual {
                Perfect => Some(Acci::NATURAL),
                Augmented(n) => Some(Acci(n as i8)),
                Diminished(n) => Some(Acci(-(n as i8))),
                _ => None,
            },
            _ => match qual {
                Major => Some(Acci::NATURAL),
                Minor => Some(Acci::FLAT),
                Augmented(n) => Some(Acci(n as i8)),
                Diminished(n) => Some(Acci(-(n as i8) - 1)),
                _ => None,
            },
        }
    }
}

impl AcciByQual for Step {
    fn acci_by_qual(&self, qual: IntervalQual) -> Option<Acci> {
        OStep::from(*self).acci_by_qual(qual)
    }
}

impl AcciByQual for SimpleIntervalDeg {
    fn acci_by_qual(&self, qual: IntervalQual) -> Option<Acci> {
        OStep::from(*self).acci_by_qual(qual)
    }
}

impl AcciByQual for IntervalDeg {
    fn acci_by_qual(&self, qual: IntervalQual) -> Option<Acci> {
        Step::from(*self).acci_by_qual(qual)
    }
}

impl From<SimpleIntervalDeg> for OStep {
    fn from(value: SimpleIntervalDeg) -> Self {
        OStep::try_from(u8::try_from(value).unwrap()).unwrap()
    }
}

impl From<IntervalDeg> for Step {
    fn from(value: IntervalDeg) -> Self {
        Step(value.0)
    }
}

impl From<SimpleInterval> for OPitch {
    fn from(value: SimpleInterval) -> OPitch {
        let step: OStep = value.deg.into();
        let tone = step.diatonic_tone() + step.acci_by_qual(value.qual).unwrap().0;
        OPitch { step, tone }
    }
}

impl From<SimpleInterval> for Pitch {
    fn from(value: SimpleInterval) -> Pitch {
        OPitch::from(value).into()
    }
}

impl From<Interval> for Pitch {
    fn from(value: Interval) -> Self {
        let step: Step = value.deg.into();
        let tone = step.diatonic_tone() + step.acci_by_qual(value.qual).unwrap().0;
        Pitch { step, tone }
    }
}

impl From<Interval> for OPitch {
    fn from(value: Interval) -> Self {
        Pitch::from(value).into()
    }
}
