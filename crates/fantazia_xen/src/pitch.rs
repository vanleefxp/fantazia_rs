use std::{
    marker::Destruct,
    ops::{Deref, Mul, MulAssign, Neg},
};

use derive_more::{Mul, MulAssign};
use fantazia_core::pitch::{Frequency, GeneralPitch, GenericPitch, SpecificPitch, ToFrequency};
#[cfg(feature = "edo12")]
use fantazia_edo12::{pitch::OPitch as OPitch12, step::OStep as OStep7};
use typed_floats::{Powf as _, tf64};

#[cfg(feature = "edo12")]
use crate::temprament::EDO_12;
// #[cfg(feature = "edo12")]
use crate::temprament::{DiatonicEDOScale, EDO12, PeriodicScaleTrait};

#[derive(Debug, Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq)]
pub struct GenericPitchWithScale<K, P> {
    pub scale: K,
    pub pitch: P,
}

const impl<K, P1, P2> GeneralPitch<GenericPitchWithScale<K, P2>> for GenericPitchWithScale<K, P1>
where
    P1: [const] GeneralPitch<P2> + [const] Destruct,
    Self: [const] Destruct,
{
    #[inline(always)]
    fn put_in_octave(self, octave: i8) -> GenericPitchWithScale<K, P2> {
        let GenericPitchWithScale { scale, pitch } = self;
        GenericPitchWithScale {
            scale,
            pitch: pitch.put_in_octave(octave),
        }
    }
}

const impl<K, P1, P2> SpecificPitch<GenericPitchWithScale<K, P2>> for GenericPitchWithScale<K, P1>
where
    P1: [const] SpecificPitch<P2> + [const] Destruct,
    Self: [const] Destruct,
{
    #[inline(always)]
    fn octave(&self) -> i8 {
        self.pitch.octave()
    }

    fn generalize(self) -> GenericPitchWithScale<K, P2> {
        let GenericPitchWithScale { scale, pitch } = self;
        GenericPitchWithScale {
            scale,
            pitch: pitch.generalize(),
        }
    }
}

const impl<K, P> PartialOrd for GenericPitchWithScale<K, P>
where
    K: [const] Eq,
    P: [const] PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.scale == other.scale {
            self.pitch.partial_cmp(&other.pitch)
        } else {
            None
        }
    }
}

// MARK: AnyOPitchEDO

#[derive(Debug, Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq)]
pub struct AnyPeriodicGeneralPitch<S = DiatonicEDOScale> {
    pub(crate) scale: S,
    pub(crate) inner: GenericPitch<u8, i16>,
}

const impl<S> AnyPeriodicGeneralPitch<S> {
    pub fn temperament(&self) -> &S {
        &self.scale
    }
    /// The nominal **step** of the pitch.
    pub fn step(&self) -> u8 {
        self.step
    }
    /// The **tone** of the pitch. An octave is divided into `self.edo()` portions and `self.tone()`
    /// denotes which portion the current pitch is on.
    pub fn tone(&self) -> i16 {
        self.tone
    }
}

const impl<S> AnyPeriodicGeneralPitch<S>
where
    S: [const] PeriodicScaleTrait + [const] Eq + [const] Destruct,
{
    /// Adds two pitches, assuming they are in the same scale.
    ///
    /// # Safety
    /// Caller must ensure that two pitches are in the same scale. Otherwise it is undefined
    /// behavior.
    pub unsafe fn add_unchecked(self, rhs: Self) -> Self {
        let Self {
            scale,
            inner: GenericPitch { step, tone },
        } = self;
        let (n_steps, n_tones) = (scale.n_steps(), scale.n_tones());

        debug_assert!(
            scale == rhs.scale,
            "two pitches not in the same scale, will cause undefined behavior"
        );

        let step = step as i16 + rhs.step as i16;
        let period = step.div_euclid(n_steps.get() as i16);
        let step = (step - period * n_steps.get() as i16) as u8;
        let tone = tone + rhs.tone - period * n_tones.get() as i16;

        Self {
            scale,
            inner: GenericPitch { step, tone },
        }
    }

    /// Subtracts another pitch from `self`, assuming they are in the same scale.
    ///
    /// # Safety
    /// Caller must ensure that two pitches are in the same scale. Otherwise it is undefined
    /// behavior.
    pub unsafe fn sub_unchecked(self, rhs: Self) -> Self {
        let Self {
            scale,
            inner: GenericPitch { step, tone },
        } = self;
        let (n_steps, n_tones) = (scale.n_steps(), scale.n_tones());

        debug_assert!(
            scale == rhs.scale,
            "two pitches not in the same scale, will cause undefined behavior"
        );
        let step = step as i16 - rhs.step as i16;
        let period = step.div_euclid(n_steps.get() as i16);
        let step = (step - period * n_steps.get() as i16) as u8;
        let tone = tone - rhs.tone - period * n_tones.get() as i16;
        Self {
            scale,
            inner: GenericPitch { step, tone },
        }
    }

    /// Adds two pitches, returns the result if they are in the same scale, or `None` if not.
    pub fn try_add(self, rhs: Self) -> Option<Self> {
        if self.scale == rhs.scale {
            // SAFETY: have checked that two pitches are in the same scale
            unsafe { Some(self.add_unchecked(rhs)) }
        } else {
            None
        }
    }

    /// Subtracts another pitch from `self`, returns the result if they are in the same scale, or
    /// `None` if not.
    pub fn try_sub(self, rhs: Self) -> Option<Self> {
        if self.scale == rhs.scale {
            // SAFETY: have checked that two pitches are in the same scale
            unsafe { Some(self.sub_unchecked(rhs)) }
        } else {
            None
        }
    }
}

const impl<S> GeneralPitch<AnyPeriodicPitch<S>> for AnyPeriodicGeneralPitch<S>
where
    S: [const] PeriodicScaleTrait + [const] Destruct,
{
    fn put_in_octave(self, octave: i8) -> AnyPeriodicPitch<S> {
        let Self {
            scale,
            inner: GenericPitch { step, tone },
        } = self;
        let (n_steps, n_tones) = (scale.n_steps(), scale.n_tones());

        let step = step as i16 + n_steps.get() as i16 * octave as i16;
        let tone = tone + n_tones.get() as i16 * octave as i16;
        AnyPeriodicPitch {
            scale,
            inner: GenericPitch { step, tone },
        }
    }
}

const impl<S> Deref for AnyPeriodicGeneralPitch<S> {
    type Target = GenericPitch<u8, i16>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

const impl<S> PartialOrd for AnyPeriodicGeneralPitch<S>
where
    S: [const] Eq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.scale
            .eq(&other.scale)
            .then(const || self.inner.cmp(&other.inner))
    }
}

impl<S> ToFrequency for AnyPeriodicGeneralPitch<S>
where
    S: PeriodicScaleTrait,
{
    fn to_frequency(&self) -> Frequency {
        let (n_tones, period) = (self.scale.n_tones(), self.scale.period());
        let p = tf64::NonNaNFinite::from(self.tone) / tf64::NonZeroNonNaNFinite::from(n_tones);
        period.powf(p)
    }
}

#[cfg(feature = "edo12")]
const impl<S> From<OPitch12> for AnyPeriodicGeneralPitch<S>
where
    S: [const] From<EDO12>,
{
    fn from(value: OPitch12) -> Self {
        Self {
            scale: EDO_12.into(),
            inner: GenericPitch {
                step: value.step as u8,
                tone: *value.tone as i16,
            },
        }
    }
}

#[cfg(feature = "edo12")]
const impl<S> TryFrom<AnyPeriodicGeneralPitch<S>> for OPitch12
where
    S: [const] From<EDO12> + [const] Eq + [const] Destruct,
{
    type Error = ();
    fn try_from(value: AnyPeriodicGeneralPitch<S>) -> Result<Self, Self::Error> {
        if value.scale == EDO_12.into() {
            // SAFETY: 12-EDO scale ensures that `value.step` is `0..7`
            let step = unsafe { OStep7::new_unchecked(value.step as i8) };
            let tone = value.tone as i8;
            Ok(OPitch12::new(step, tone))
        } else {
            Err(())
        }
    }
}

const impl<S> Mul<i8> for AnyPeriodicGeneralPitch<S>
where
    S: [const] PeriodicScaleTrait,
{
    type Output = Self;
    fn mul(mut self, rhs: i8) -> Self::Output {
        self *= rhs;
        self
    }
}

const impl<S> MulAssign<i8> for AnyPeriodicGeneralPitch<S>
where
    S: [const] PeriodicScaleTrait,
{
    fn mul_assign(&mut self, rhs: i8) {
        let Self {
            ref scale,
            inner: GenericPitch { step, tone },
        } = *self;
        let (n_steps, n_tones) = (scale.n_steps(), scale.n_tones());
        let step = step as i32 * rhs as i32;
        let period = step.div_euclid(n_steps.get() as i32);
        self.inner.step = (step - period * n_steps.get() as i32) as u8;
        self.inner.tone = (tone as i32 * rhs as i32 - period * n_tones.get() as i32) as i16;
    }
}

const impl<S> Neg for AnyPeriodicGeneralPitch<S>
where
    S: [const] PeriodicScaleTrait + [const] Destruct,
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        let Self {
            scale,
            inner: GenericPitch { step, tone },
        } = self;
        let (n_steps, n_tones) = (scale.n_steps(), scale.n_tones());
        if step == 0 {
            Self {
                scale,
                inner: GenericPitch {
                    step: 0,
                    tone: -tone,
                },
            }
        } else {
            let step = n_steps.get() - step;
            let tone = n_tones.get() as i16 - tone;
            Self {
                scale,
                inner: GenericPitch { step, tone },
            }
        }
    }
}

// MARK: AnyPitchEDO
#[derive(Debug, Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq, Mul, MulAssign)]
#[const_impl]
pub struct AnyPeriodicPitch<S> {
    #[mul(skip)]
    pub(crate) scale: S,
    pub(crate) inner: GenericPitch<i16, i16>,
}

const impl<S> AnyPeriodicPitch<S>
where
    S: [const] Eq + [const] Destruct,
{
    pub unsafe fn add_unchecked(mut self, rhs: Self) -> Self {
        debug_assert!(self.scale == rhs.scale);
        self.inner += rhs.inner;
        self
    }
    pub unsafe fn sub_unchecked(mut self, rhs: Self) -> Self {
        debug_assert!(self.scale == rhs.scale);
        self.inner -= rhs.inner;
        self
    }
    pub fn try_add(self, rhs: Self) -> Option<Self> {
        if self.scale == rhs.scale {
            // SAFETY: have checked that two pitches are in the same scale
            unsafe { Some(self.add_unchecked(rhs)) }
        } else {
            None
        }
    }
    pub fn try_sub(self, rhs: Self) -> Option<Self> {
        if self.scale == rhs.scale {
            // SAFETY: have checked that two pitches are in the same scale
            unsafe { Some(self.sub_unchecked(rhs)) }
        } else {
            None
        }
    }
}

const impl<S> PartialOrd for AnyPeriodicPitch<S>
where
    S: [const] Eq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.scale
            .eq(&other.scale)
            .then(const || self.inner.cmp(&other.inner))
    }
}

const impl<S> SpecificPitch<AnyPeriodicGeneralPitch<S>> for AnyPeriodicPitch<S>
where
    S: [const] PeriodicScaleTrait + [const] Destruct,
{
    fn octave(&self) -> i8 {
        self.inner
            .step
            .div_euclid(self.scale.n_steps().get() as i16) as i8
    }
    fn generalize(self) -> AnyPeriodicGeneralPitch<S> {
        let octave = self.octave();
        let Self {
            scale,
            inner: GenericPitch { step, tone },
        } = self;
        let (n_steps, n_tones) = (scale.n_steps(), scale.n_tones());
        let step = (step - octave as i16 * n_steps.get() as i16) as u8;
        let tone = tone - octave as i16 * n_tones.get() as i16;
        AnyPeriodicGeneralPitch {
            scale,
            inner: GenericPitch { step, tone },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_any_pitch_edo_mul() {
        let perfect_fifth_edo53 = DiatonicEDOScale::EDO_53.new_pitch(4, 31).generalize();

        println!("{:?}", DiatonicEDOScale::EDO_53);
        println!("{perfect_fifth_edo53:#?}");
        println!("{}", perfect_fifth_edo53.to_frequency());

        for i in -1i8..11 {
            let pitch = perfect_fifth_edo53 * i;
            println!("{:?}", pitch);
        }
    }
}
