use anyhow::bail;
use derive_more::*;
use malachite_base::num::arithmetic::traits::{DivMod as _, Mod as _};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use super::base::{Acci, OPitch, OStep, Pitch, Step};
use crate::{impl_from_mod, traits::FromMod};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum IntervalQual {
    Diminished(u8),
    Minor,
    Perfect,
    Major,
    Augmented(u8),
}

#[repr(u8)]
#[derive(IntoPrimitive, TryFromPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum OIntervalDeg {
    Unison = 0,
    Second = 1,
    Third = 2,
    Fourth = 3,
    Fifth = 4,
    Sixth = 5,
    Seventh = 6,
}

impl_from_mod!(OIntervalDeg, 7, u8; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

impl From<OStep> for OIntervalDeg {
    fn from(value: OStep) -> Self {
        OIntervalDeg::try_from(u8::try_from(value).unwrap()).unwrap()
    }
}

impl From<Step> for OIntervalDeg {
    fn from(value: Step) -> Self {
        IntervalDeg::from(value).into()
    }
}

impl From<IntervalDeg> for OIntervalDeg {
    fn from(value: IntervalDeg) -> Self {
        Self::from_mod(value.0)
    }
}

#[derive(
    From,
    Into,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    Neg,
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
    pub fn from_odeg_and_octave(odeg: OIntervalDeg, octave: i8) -> Self {
        IntervalDeg(odeg as i8 + octave * 7)
    }

    pub fn octave(&self) -> i8 {
        self.0.div_mod(7).0
    }

    pub fn into_odeg_and_octave(&self) -> (OIntervalDeg, i8) {
        let (octave, odeg) = self.0.div_mod(7);
        let odeg = odeg as u8;
        (OIntervalDeg::try_from(odeg).unwrap(), octave)
    }
}

impl From<Step> for IntervalDeg {
    fn from(value: Step) -> Self {
        IntervalDeg(value.0)
    }
}

impl From<OStep> for IntervalDeg {
    fn from(value: OStep) -> Self {
        OIntervalDeg::from(value).into()
    }
}

impl From<OIntervalDeg> for IntervalDeg {
    fn from(value: OIntervalDeg) -> Self {
        IntervalDeg(value as i8)
    }
}

pub trait Qual {
    fn qual(&self) -> IntervalQual;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Neg)]
pub struct OInterval {
    pub(crate) deg: OIntervalDeg,
    pub(crate) qual: IntervalQual,
}

impl OInterval {
    pub fn try_from_deg_and_qual(deg: OIntervalDeg, qual: IntervalQual) -> Result<Self, anyhow::Error> {
        use OIntervalDeg::*;
        use IntervalQual::*;
        match (qual, deg) {
            (Major | Minor, Unison | Fourth | Fifth)
            | (Perfect, Second | Third | Sixth | Seventh) => {
                bail!("{qual}{deg} is not a valid simple interval.")
            }
            _ => Ok(OInterval { deg, qual })
        }
    }

    pub fn from_deg_and_qual(deg: OIntervalDeg, qual: IntervalQual) -> Self {
        Self::try_from_deg_and_qual(deg, qual).unwrap()
    }

    pub const unsafe fn from_deg_and_qual_unchecked(deg: OIntervalDeg, qual: IntervalQual) -> Self {
        OInterval { deg, qual }
    }

    pub fn deg(&self) -> OIntervalDeg {
        self.deg
    }
}

impl Qual for OInterval {
    fn qual(&self) -> IntervalQual {
        self.qual
    }
}

impl From<Interval> for OInterval {
    fn from(value: Interval) -> Self {
        OInterval {
            deg: value.deg.into(),
            qual: value.qual,
        }
    }
}

impl From<OPitch> for OInterval {
    fn from(value: OPitch) -> Self {
        let qual = value.qual();
        let deg = OIntervalDeg::from(value.step);
        OInterval { deg, qual }
    }
}

impl From<Pitch> for OInterval {
    fn from(value: Pitch) -> Self {
        OPitch::from(value).into()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Interval {
    pub(crate) deg: IntervalDeg,
    pub(crate) qual: IntervalQual,
}

impl Interval {
    pub fn from_ointerval_and_octave(ointerval: OInterval, octave: i8) -> Self {
        let deg = (ointerval.deg as i8 + 7 * octave).into();
        let qual = ointerval.qual;
        Interval { deg, qual }
    }

    pub fn from_deg_and_qual(deg: IntervalDeg, qual: IntervalQual) -> Result<Self, anyhow::Error> {
        use OIntervalDeg::*;
        use IntervalQual::*;
        match (qual, OIntervalDeg::from(deg)) {
            (Major | Minor, Unison | Fourth | Fifth)
            | (Perfect, Second | Third | Sixth | Seventh) => {
                bail!("Not a valid interval.")
            }
            _ => Ok(Interval { deg, qual })
        }
    }

    pub const unsafe fn from_deg_and_qual_unchecked(deg: IntervalDeg, qual: IntervalQual) -> Self {
        Interval { deg, qual }
    }

    pub fn deg(&self) -> IntervalDeg {
        self.deg
    }

    pub fn odeg(&self) -> OIntervalDeg {
        self.deg.into()
    }

    pub fn octave(&self) -> i8 {
        self.deg.octave()
    }
}

impl From<OInterval> for Interval {
    fn from(value: OInterval) -> Self {
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

impl AcciByQual for OIntervalDeg {
    fn acci_by_qual(&self, qual: IntervalQual) -> Option<Acci> {
        OStep::from(*self).acci_by_qual(qual)
    }
}

impl AcciByQual for IntervalDeg {
    fn acci_by_qual(&self, qual: IntervalQual) -> Option<Acci> {
        Step::from(*self).acci_by_qual(qual)
    }
}

impl From<OIntervalDeg> for OStep {
    fn from(value: OIntervalDeg) -> Self {
        OStep::try_from(u8::try_from(value).unwrap()).unwrap()
    }
}

impl From<IntervalDeg> for Step {
    fn from(value: IntervalDeg) -> Self {
        Step(value.0)
    }
}

impl From<OInterval> for OPitch {
    fn from(value: OInterval) -> OPitch {
        let step: OStep = value.deg.into();
        let tone = step.diatonic_tone() + step.acci_by_qual(value.qual).unwrap().0;
        OPitch { step, tone }
    }
}

impl From<OInterval> for Pitch {
    fn from(value: OInterval) -> Pitch {
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
