use derive_more::{Add, AddAssign, Deref, DerefMut, From, Into, Neg, Sub, SubAssign, Sum};
use malachite_base::num::arithmetic::traits::{DivMod, EqMod, Mod};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uncased::UncasedStr;
use crate::{impl_from_mod, traits::FromMod};

pub(crate) const DIATONIC: [i8; 7] = [0, 2, 4, 5, 7, 9, 11];

pub(crate) static STEP_NAMES: phf::Map<&UncasedStr, OStep> = phf::phf_map! {
    UncasedStr::new("C") | UncasedStr::new("do") | UncasedStr::new("ut") => OStep::C,
    UncasedStr::new("D") | UncasedStr::new("re") => OStep::D,
    UncasedStr::new("E") | UncasedStr::new("mi") => OStep::E,
    UncasedStr::new("F") | UncasedStr::new("fa") => OStep::F,
    UncasedStr::new("G") | UncasedStr::new("sol") => OStep::G,
    UncasedStr::new("A") | UncasedStr::new("la") => OStep::A,
    UncasedStr::new("B") | UncasedStr::new("si") | UncasedStr::new("ti") => OStep::B,
};

#[repr(u8)]
#[derive(
    IntoPrimitive, TryFromPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default,
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum OStep {
    #[default]
    C = 0,
    D = 1,
    E = 2,
    F = 3,
    G = 4,
    A = 5,
    B = 6,
}

impl OStep {
    #[inline]
    pub const fn diatonic_tone(&self) -> i8 {
        DIATONIC[(*self) as usize]
    }
}

impl From<Step> for OStep {
    fn from(value: Step) -> Self {
        Self::from_mod(value.0)
    }
}

#[derive(
    From,
    Into,
    Add,
    AddAssign,
    Sum,
    Sub,
    SubAssign,
    Neg,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    PartialOrd,
    Ord,
    Hash,
    Deref,
    DerefMut,
    Default,
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Step(pub i8);

impl Step {
    pub fn from_ostep_and_octave(ostep: OStep, octave: i8) -> Self {
        Step(ostep as i8 + 7 * octave)
    }

    pub fn octave(&self) -> i8 {
        self.0.div_mod(7).0
    }

    pub fn ostep_and_octave(self) -> (OStep, i8) {
        let (octave, ostep) = self.0.div_mod(7);
        (OStep::try_from(ostep as u8).unwrap(), octave)
    }

    pub fn diatonic_tone(self) -> i8 {
        let (octave, ostep) = self.0.div_mod(7);
        DIATONIC[ostep as usize] + 12 * octave
    }
}

impl From<OStep> for Step {
    fn from(value: OStep) -> Self {
        Self(value as i8)
    }
}

impl_from_mod!(OStep, 7, u8; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

#[derive(
    From,
    Into,
    Add,
    AddAssign,
    Sum,
    Sub,
    SubAssign,
    Neg,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    PartialOrd,
    Ord,
    Hash,
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Acci(pub(crate) i8);
// in real music it is not common to use accidentals that modifies a pitch by more than 2 semitones
// so `i8` would be enough

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct OPitch {
    pub step: OStep,
    pub tone: i8,
}

impl OPitch {
    pub const fn new(step: OStep, acci: Acci) -> Self {
        let tone = step.diatonic_tone() + acci.0;
        OPitch { step, tone }
    }

    pub const fn from_step_and_tone(step: OStep, tone: i8) -> Self {
        OPitch { step, tone }
    }

    pub fn acci(&self) -> Acci {
        (self.tone as i8 - self.step.diatonic_tone() as i8).into()
    }

    pub fn is_diatonic(&self) -> bool {
        self.tone == self.step.diatonic_tone()
    }

    pub fn is_enharmonic(&self, other: &Self) -> bool {
        self.tone.eq_mod(other.tone, 12)
    }
}

impl From<OStep> for OPitch {
    /// Creates a diatonic pitch.
    fn from(value: OStep) -> Self {
        OPitch {
            step: value,
            tone: value.diatonic_tone(),
        }
    }
}

impl From<Pitch> for OPitch {
    /// Creates a pitch in central octave.
    fn from(pitch: Pitch) -> Self {
        let (ostep, octave) = pitch.step.ostep_and_octave();
        let otone = pitch.tone - 12 * octave;
        OPitch {
            step: ostep,
            tone: otone,
        }
    }
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    PartialOrd,
    Ord,
    Add,
    AddAssign,
    Sum,
    Sub,
    SubAssign,
    Neg,
    Hash,
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Pitch {
    pub step: Step,
    pub tone: i8,
}

impl Pitch {
    pub const fn from_step_and_tone(step: Step, tone: i8) -> Self {
        Pitch { step, tone }
    }

    pub fn from_step_and_acci(step: Step, acci: Acci) -> Self {
        let tone = step.diatonic_tone() + acci.0;
        Pitch { step, tone }
    }

    pub const fn from_opitch_and_octave(opitch: OPitch, octave: i8) -> Self {
        let step = Step(opitch.step as i8 + 7 * octave);
        let tone = opitch.tone + 12 * octave;
        Pitch { step, tone }
    }

    pub fn opitch_and_octave(self) -> (OPitch, i8) {
        let (ostep, octave) = self.step.ostep_and_octave();
        let otone = self.tone - 12 * octave;
        (
            OPitch {
                step: ostep,
                tone: otone,
            },
            octave,
        )
    }

    pub fn acci(&self) -> Acci {
        (self.tone - self.step.diatonic_tone()).into()
    }

    pub fn is_diatonic(&self) -> bool {
        self.tone == self.step.diatonic_tone()
    }

    pub fn is_enharmonic(&self, other: &Self) -> bool {
        self.tone == other.tone
    }
}

impl From<OPitch> for Pitch {
    fn from(opitch: OPitch) -> Self {
        Pitch {
            step: opitch.step.into(),
            tone: opitch.tone,
        }
    }
}
