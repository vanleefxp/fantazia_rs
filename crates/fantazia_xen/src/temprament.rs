use std::{
    borrow::Cow,
    cell, fmt,
    num::NonZeroU8,
    rc::Rc,
    sync::{self, Arc},
};

use derive_more::{Debug, Deref, DerefMut, Display, From, Into};
use fantazia_core::pitch::GenericPitch;
use fantazia_edo12::step::Step as Step7;
use paste::paste;
use typed_floats::tf64;

use crate::pitch::{AnyPeriodicGeneralPitch, AnyPeriodicPitch};

macro_rules! tf_helper {
    (($value:expr) as $t:ty) => {
        const { <$t>::new($value).ok().unwrap() }
    };
}

macro_rules! nonzero {
    ($value:expr) => {
        const { core::num::NonZero::new($value).unwrap() }
    };
}

macro_rules! tf {
    (+$value:literal as $t:ty) => {
        tf_helper!((+$value) as $t)
    };
    (-$value:literal as $t:ty) => {
        tf_helper!((-$value) as $t)
    };
    ($value:literal as $t:ty) => {
        tf_helper!(($value) as $t)
    };
}

/// Creates a unit struct that represents a constant of a specific type.
/// This can resolve the gap that only a limited range of types are supported in generic constants.
macro_rules! const_unit_struct {
    {$(
        $(#[$attr:meta])*
        $vis:vis struct $name:ident : $type:ty = $value:expr;
    )*} => {
        $($(#[$attr])*
        #[derive(Copy, Hash)]
        #[derive_const(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
        $vis struct $name;

        const impl From<$name> for $type {
            fn from(_value: $name) -> Self {
                $value
            }
        })*
    };
}

type PeriodRepr = tf64::StrictlyPositiveFinite;
type NStepsRepr = NonZeroU8;
type NTonesRepr = NonZeroU8;

const_unit_struct! {
    /// An octave-based period, where the frequency ratio of two neighboring periods is 2.
    #[derive(Debug)]
    pub struct Octave: PeriodRepr = tf!(2. as PeriodRepr);

    /// A diatonic scale, with exactly 7 steps.
    #[derive(Debug)]
    pub struct Diatonic: NStepsRepr = nonzero!(7);

    /// A scale with exactly 12 tones.
    #[derive(Debug)]
    pub struct _12Tones: NTonesRepr = nonzero!(12);
}

#[derive(Debug, Display, Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq, PartialOrd, Ord, From, Into, Deref, DerefMut)]
#[const_impl]
#[debug("Period({})", self.0)]
#[display("{}", self.0)]
pub struct Period(PeriodRepr);

const impl Default for Period {
    fn default() -> Self {
        Octave.into()
    }
}

const impl From<Octave> for Period {
    fn from(value: Octave) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Display, Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq, PartialOrd, Ord, From, Into, Deref, DerefMut)]
#[const_impl]
/// A scale that may contain an arbitrary number of steps.
pub struct NSteps(NStepsRepr);

const impl Default for NSteps {
    fn default() -> Self {
        Diatonic.into()
    }
}

const impl From<Diatonic> for NSteps {
    fn from(value: Diatonic) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Display, Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq, PartialOrd, Ord, From, Into, Deref, DerefMut)]
#[const_impl]
/// A scale that may contain an arbitrary number of tones.
pub struct NTones(NTonesRepr);

const impl Default for NTones {
    fn default() -> Self {
        _12Tones.into()
    }
}

const impl From<_12Tones> for NTones {
    fn from(_value: _12Tones) -> Self {
        Self(_12Tones.into())
    }
}

/// Trait representing a periodic scale. This is needed for references of periodic scales to be seen as
/// periodic scales.
pub const trait PeriodicScaleTrait {
    /// Number of nominal steps.
    fn n_steps(&self) -> NStepsRepr;

    /// Number of divisions per frequency period.
    fn n_tones(&self) -> NTonesRepr;

    /// The frequency period. i.e. The frequency ratio between the higher period and the lower
    /// neighboring period. For example, the frequency period of octave-based scales are 2.
    fn period(&self) -> PeriodRepr;

    fn new_pitch(&self, step: i16, tone: i16) -> AnyPeriodicPitch<Self>
    where
        Self: Copy + Sized,
    {
        AnyPeriodicPitch {
            scale: *self,
            inner: GenericPitch { step, tone },
        }
    }

    unsafe fn new_general_pitch_unchecked(
        &self,
        step: u8,
        tone: i16,
    ) -> AnyPeriodicGeneralPitch<Self>
    where
        Self: Copy + Sized,
    {
        AnyPeriodicGeneralPitch {
            scale: *self,
            inner: GenericPitch { step, tone },
        }
    }
}

macro_rules! impl_periodic_scale_ref {
    ($($t:ty),+$(,)?) => {
        $(impl<T: PeriodicScaleTrait + ?Sized> PeriodicScaleTrait for $t {
            fn n_steps(&self) -> NStepsRepr {
                (**self).n_steps()
            }
            fn n_tones(&self) -> NTonesRepr {
                (**self).n_tones()
            }
            fn period(&self) -> PeriodRepr {
                (**self).period()
            }
        })+
    };
}

impl_periodic_scale_ref!(
    &T,
    &mut T,
    Box<T>,
    Rc<T>,
    Arc<T>,
    cell::Ref<'_, T>,
    cell::RefMut<'_, T>,
    sync::MutexGuard<'_, T>,
    sync::MappedMutexGuard<'_, T>,
    sync::RwLockReadGuard<'_, T>,
    sync::RwLockWriteGuard<'_, T>,
    sync::MappedRwLockReadGuard<'_, T>,
    sync::MappedRwLockWriteGuard<'_, T>,
);

impl<T> PeriodicScaleTrait for Cow<'_, T>
where
    T: PeriodicScaleTrait + ?Sized + ToOwned,
    T::Owned: PeriodicScaleTrait,
{
    fn n_steps(&self) -> NonZeroU8 {
        match self {
            Cow::Borrowed(v) => (**v).n_steps(),
            Cow::Owned(v) => v.n_steps(),
        }
    }
    fn n_tones(&self) -> NonZeroU8 {
        match self {
            Cow::Borrowed(v) => (**v).n_tones(),
            Cow::Owned(v) => v.n_tones(),
        }
    }
    fn period(&self) -> PeriodRepr {
        match self {
            Cow::Borrowed(v) => (**v).period(),
            Cow::Owned(v) => v.period(),
        }
    }
}

#[derive(Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq)]
pub struct PeriodicScale<S = NSteps, T = NTones, P = Period> {
    pub n_steps: S,
    pub n_tones: T,
    pub period: P,
}

const impl<S, T, P> PeriodicScaleTrait for PeriodicScale<S, T, P>
where
    S: [const] Into<NSteps> + Copy,
    T: [const] Into<NTones> + Copy,
    P: [const] Into<Period> + Copy,
{
    fn n_steps(&self) -> NStepsRepr {
        *self.n_steps.into()
    }
    fn n_tones(&self) -> NTonesRepr {
        *self.n_tones.into()
    }
    fn period(&self) -> PeriodRepr {
        *self.period.into()
    }
}

const impl<S, T, P> Default for PeriodicScale<S, T, P>
where
    Self: [const] From<EDO12>,
{
    fn default() -> Self {
        Self::from(EDO_12)
    }
}

impl<S, T, P> fmt::Debug for PeriodicScale<S, T, P>
where
    Self: PeriodicScaleTrait,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PeriodicScale(\"{}\")", self)
    }
}

impl<S, T, P> fmt::Display for PeriodicScale<S, T, P>
where
    Self: PeriodicScaleTrait,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n_steps = self.n_steps();
        let n_tones = self.n_tones();
        let period = self.period();
        write!(f, "{}_{}ed{}", n_steps, n_tones, period)
    }
}

/// A scale that divides the octave into any number of steps and tones.
/// This is an octave-based scale so it always has a frequency period of 2.
pub type EDOScale<S = NSteps, T = NTones> = PeriodicScale<S, T, Octave>;

impl EDOScale {
    #[inline(always)]
    pub const fn new(
        n_steps: impl [const] Into<NSteps>,
        n_tones: impl [const] Into<NTones>,
    ) -> Self {
        Self {
            n_steps: n_steps.into(),
            n_tones: n_tones.into(),
            period: Octave,
        }
    }
}

/// A scale that divides the octave into 7 steps and an arbitrary number of tones.
pub type DiatonicEDOScale<T = NTones> = EDOScale<Diatonic, T>;
const LOG_2_3_M1: f64 = 0.58496250072115618667822900533792562782764434814453125;

macro_rules! gen_edo_const {
    ($($n:literal),*$(,)?) => {
        $(paste! {
            pub const [<EDO_ $n>]: Self = Self::new(nonzero!($n));
        })*
    };
}

const impl DiatonicEDOScale {
    gen_edo_const!(19, 31, 41, 53);

    #[inline(always)]
    pub fn new(n_tones: impl [const] Into<NTones>) -> Self {
        Self {
            n_steps: Diatonic,
            n_tones: n_tones.into(),
            period: Octave,
        }
    }
}

const impl<T> DiatonicEDOScale<T>
where
    T: [const] Into<NTones> + Copy,
{
    /// Determines how many steps a perfect fifth contains.
    pub fn fifth_size(&self) -> NonZeroU8 {
        let n_tones = self.n_tones();
        // `LOG_2_3_M1` is approx. 0.584 so the rounding result is at least 1 and should never
        // overflow
        unsafe { NonZeroU8::new_unchecked((n_tones.get() as f64 * LOG_2_3_M1).round() as u8) }
    }

    pub fn perfect_fifth(&self) -> AnyPeriodicGeneralPitch<Self> {
        AnyPeriodicGeneralPitch {
            scale: *self,
            inner: GenericPitch {
                step: 4,
                tone: self.fifth_size().get() as i16,
            },
        }
    }

    pub fn new_diatonic(&self, step: Step7) -> AnyPeriodicGeneralPitch<Self> {
        self.perfect_fifth() * step.co5_order()
    }
}

pub type EDO12 = DiatonicEDOScale<_12Tones>;
pub const EDO_12: EDO12 = EDO12 {
    n_steps: Diatonic,
    n_tones: _12Tones,
    period: Octave,
};

macro_rules! impl_scale_conversion {
    () => {};
    ($($from:ty,$to:ty);+$(;)?) => {
        $(const impl From<$from> for $to {
            fn from(value: $from) -> Self {
                let PeriodicScale {
                    n_steps,
                    n_tones,
                    period,
                } = value;
                #[allow(clippy::useless_conversion)] // code generated by macro
                Self {
                    n_steps: n_steps.into(),
                    n_tones: n_tones.into(),
                    period: period.into(),
                }
            }
        })+
    };
}

impl_scale_conversion!(
    EDO12, DiatonicEDOScale;
    DiatonicEDOScale, EDOScale;
    EDOScale, PeriodicScale;
    EDO12, EDOScale;
    EDO12, PeriodicScale;
    DiatonicEDOScale, PeriodicScale;
);
