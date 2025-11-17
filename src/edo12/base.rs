use derive_more::{Add, AddAssign, From, Into, Neg, Sub, SubAssign};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uncased::UncasedStr;

use crate::traits::Co5Order;

pub(crate) const DIATONIC: [i8; 7] = [0, 2, 4, 5, 7, 9, 11];

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

#[derive(Clone, Copy, PartialEq, Eq, From, Into, Add, AddAssign, Sub, SubAssign, Neg)]
pub struct Acci(pub(crate) i8);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OPitch {
    pub(crate) step: Step,
    pub(crate) tone: i8,
}

impl OPitch {
    pub fn new(step: Step, acci: Acci) -> Self {
        let tone = DIATONIC[step as usize] + acci.0;
        OPitch { step, tone }
    }

    pub fn from_step_and_tone(step: Step, tone: i8) -> Self {
        OPitch { step, tone }
    }

    pub fn step(&self) -> Step {
        self.step
    }

    pub fn tone(&self) -> i8 {
        self.tone
    }

    pub fn acci(&self) -> Acci {
        (self.tone as i8 - DIATONIC[self.step as usize] as i8).into()
    }

    pub fn is_diatonic(&self) -> bool {
        self.tone == DIATONIC[self.step as usize]
    }
}


macro_rules! impl_co5_order_as_primitive {
    ($($t:ty),+ $(,)?) => {
        $(
            impl Co5Order<$t> for OPitch {
                fn co5_order(self) -> $t {
                    self.step as $t * 2 % 7 + <$t>::from(self.acci()) * 7
                }
            }
        )*
    };
}

impl_co5_order_as_primitive!(i8);

