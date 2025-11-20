use derive_more::{Add, AddAssign, From, Into, Neg, Sub, SubAssign, Sum};
use malachite_base::num::arithmetic::traits::{DivMod, Mod, ModMul};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uncased::UncasedStr;

use crate::traits::Co5Order;

pub(crate) const DIATONIC: [i8; 7] = [0, 2, 4, 5, 7, 9, 11];
pub(crate) const CO5_ORDER: [i8; 7] = [0, 2, 4, -1, 1, 3, 5];

pub(crate) static STEP_NAMES: phf::Map<&UncasedStr, Step> = phf::phf_map! {
    UncasedStr::new("C") | UncasedStr::new("do") | UncasedStr::new("ut") => Step::C,
    UncasedStr::new("D") | UncasedStr::new("re") => Step::D,
    UncasedStr::new("E") | UncasedStr::new("mi") => Step::E,
    UncasedStr::new("F") | UncasedStr::new("fa") => Step::F,
    UncasedStr::new("G") | UncasedStr::new("sol") => Step::G,
    UncasedStr::new("A") | UncasedStr::new("la") => Step::A,
    UncasedStr::new("B") | UncasedStr::new("si") | UncasedStr::new("ti") => Step::B,
};

#[repr(u8)]
#[derive(IntoPrimitive, TryFromPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Step {
    C = 0,
    D = 1,
    E = 2,
    F = 3,
    G = 4,
    A = 5,
    B = 6,
}

impl Step {
    #[inline]
    pub const fn diatonic_tone(&self) -> i8 {
        DIATONIC[(*self) as usize]
    }
}

// macro_rules! impl_step_from_primitive {
//     ($($t:ty),+$(,)?) => {
//         $(
//             impl From<$t> for Step {
//                 fn from(value: $t) -> Self {
//                     Step::try_from((value % 7) as u8).unwrap()
//                 }
//             }
//         )*
//     };
// }

// impl_step_from_primitive!(i8, u16, i16, u32, i32, u64, i64, u128, i128);

#[derive(Clone, Copy, PartialEq, Eq, From, Into, Add, AddAssign, Sub, SubAssign, Neg, Sum)]
pub struct Acci(pub(crate) i8);
// in real music it is not common to use accidentals that modifies a pitch by more than 2 semitones
// so `i8` would be enough

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OPitch {
    pub(crate) step: Step,
    pub(crate) tone: i8,
}

impl OPitch {
    pub const fn new(step: Step, acci: Acci) -> Self {
        let tone = step.diatonic_tone() + acci.0;
        OPitch { step, tone }
    }

    pub const fn new_diatonic(step: Step) -> Self {
        OPitch { step, tone: step.diatonic_tone() }
    }

    pub const fn from_step_and_tone(step: Step, tone: i8) -> Self {
        OPitch { step, tone }
    }

    pub fn from_co5_order(co5_order: i8) -> Self {
        let step: Step = (co5_order.mod_op(7) as u8)
            .mod_mul(4, 7)
            .try_into()
            .unwrap();
        let tone = step.diatonic_tone() + (co5_order + 1).div_mod(7).0;
        OPitch { step, tone }
    }

    pub const fn step(&self) -> Step {
        self.step
    }

    pub const fn tone(&self) -> i8 {
        self.tone
    }

    pub fn acci(&self) -> Acci {
        (self.tone as i8 - self.step.diatonic_tone() as i8).into()
    }

    pub const fn is_diatonic(&self) -> bool {
        self.tone == self.step.diatonic_tone()
    }

    pub const fn is_enharmonic(&self, other: &Self) -> bool {
        self.tone == other.tone
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
