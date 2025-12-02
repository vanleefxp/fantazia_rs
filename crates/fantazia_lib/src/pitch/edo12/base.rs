use derive_more::{Add, AddAssign, From, Into, Neg, Sub, SubAssign, Sum};
use malachite_base::num::{
    arithmetic::traits::{DivMod, EqMod, Mod, ModMul},
    basic::traits::Zero,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uncased::UncasedStr;

use crate::{
    impl_from_mod,
    traits::{Co5Order, FromMod},
};

pub(crate) const DIATONIC: [i8; 7] = [0, 2, 4, 5, 7, 9, 11];
pub(crate) const CO5_ORDER: [i8; 7] = [0, 2, 4, -1, 1, 3, 5];

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
#[derive(IntoPrimitive, TryFromPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum OStep {
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
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Step(pub(crate) i8);

impl Step {
    pub fn octave(&self) -> i8 {
        self.0.div_mod(7).0
    }

    pub fn into_ostep_and_octave(self) -> (OStep, i8) {
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

pub trait PitchNotation {
    type Step;
    type OStep;
    type Acci: Zero + PartialEq;
    type OTone;
    type Tone: PartialEq;
    type Octave;

    fn step(&self) -> Self::Step;
    fn tone(&self) -> Self::Tone;
    fn ostep(&self) -> Self::OStep;
    fn octave(&self) -> Self::Octave;
    fn octave_by_tone(&self) -> Self::Octave;
    fn otone(&self) -> Self::OTone;
    fn acci(&self) -> Self::Acci;

    fn is_diatonic(&self) -> bool {
        self.acci() == Self::Acci::ZERO
    }

    fn is_enharmonic(&self, other: &Self) -> bool {
        self.tone() == other.tone()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature="rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct OPitch {
    pub(crate) step: OStep,
    pub(crate) tone: i8,
}

impl PitchNotation for OPitch {
    type Step = OStep;
    type OStep = OStep;
    type Acci = Acci;
    type OTone = i8;
    type Tone = i8;
    type Octave = i8;

    fn step(&self) -> Self::Step {
        self.step
    }

    fn tone(&self) -> Self::Tone {
        self.tone
    }

    fn ostep(&self) -> Self::OStep {
        self.step
    }

    fn octave(&self) -> Self::Octave {
        0
    }

    fn octave_by_tone(&self) -> Self::Octave {
        self.tone.div_mod(12).0
    }

    fn otone(&self) -> Self::OTone {
        self.tone
    }

    fn acci(&self) -> Self::Acci {
        (self.tone as i8 - self.step.diatonic_tone() as i8).into()
    }

    fn is_diatonic(&self) -> bool {
        self.tone == self.step.diatonic_tone()
    }

    fn is_enharmonic(&self, other: &Self) -> bool {
        self.tone.eq_mod(other.tone, 12)
    }
}

impl OPitch {
    pub const fn new(step: OStep, acci: Acci) -> Self {
        let tone = step.diatonic_tone() + acci.0;
        OPitch { step, tone }
    }

    pub const fn from_step_and_tone(step: OStep, tone: i8) -> Self {
        OPitch { step, tone }
    }

    pub fn from_co5_order(co5_order: i8) -> Self {
        let step: OStep = (co5_order.mod_op(7) as u8)
            .mod_mul(4, 7)
            .try_into()
            .unwrap();
        let tone = step.diatonic_tone() + (co5_order + 1).div_mod(7).0;
        OPitch { step, tone }
    }
}

impl From<OStep> for OPitch {
    fn from(value: OStep) -> Self {
        OPitch {
            step: value,
            tone: value.diatonic_tone(),
        }
    }
}

impl From<Pitch> for OPitch {
    fn from(pitch: Pitch) -> Self {
        let (ostep, octave) = pitch.step.into_ostep_and_octave();
        let otone = pitch.tone - 12 * octave;
        OPitch {
            step: ostep,
            tone: otone,
        }
    }
}

macro_rules! impl_co5_order_as_primitive {
    ($($t:ty),+ $(,)?) => {
        $(
            impl Co5Order<$t> for OPitch {
                fn co5_order(self) -> $t {
                    CO5_ORDER[self.step as usize] as $t + <$t>::from(self.acci()) * 7
                }
            }
        )*
    };
}

impl_co5_order_as_primitive!(i8);

#[derive(
    Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord, Add, AddAssign, Sum, Sub, SubAssign, Neg, Hash,
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Pitch {
    pub(crate) step: Step,
    pub(crate) tone: i8,
}

impl PitchNotation for Pitch {
    type Step = Step;
    type OStep = OStep;
    type Acci = Acci;
    type OTone = i8;
    type Tone = i8;
    type Octave = i8;

    fn step(&self) -> Step {
        self.step
    }

    fn tone(&self) -> i8 {
        self.tone
    }

    fn ostep(&self) -> OStep {
        self.step.into()
    }

    fn octave(&self) -> i8 {
        self.step.octave()
    }

    fn octave_by_tone(&self) -> i8 {
        self.tone.div_mod(12).0
    }

    fn otone(&self) -> i8 {
        self.tone - 12 * self.octave()
    }

    fn acci(&self) -> Acci {
        (self.tone - self.step.diatonic_tone()).into()
    }
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

    pub fn into_opitch_and_octave(self) -> (OPitch, i8) {
        let (ostep, octave) = self.step.into_ostep_and_octave();
        let otone = self.tone - 12 * octave;
        (
            OPitch {
                step: ostep,
                tone: otone,
            },
            octave,
        )
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
