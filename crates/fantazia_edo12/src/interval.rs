//! Interval representation of pitches.
//!
//! This module provides:
//! - the [`IntervalQual`] type representing interval qualities (major, minor, perfect, etc.).
//! - the [`OInterval`] type for general intervals not considering the number of octaves crossed.
//! - the [`Interval`] type for specific intervals that may cross multiple octaves.
//!
//! [`OInterval`] and [`Interval`] are convertible and fully equivalent to [`OPitch`] and [`Pitch`],
//! respectively, but they keep track of the _interval quality_ and number of steps instead of _tone_ and
//! step. So it's suggested to use [`OPitch`] and [`Pitch`] for pitch / interval arithmetics instead.
//! Only use the corresponding interval types when you need to work with interval representation.

use derive_more::{From, Hash, Into, Neg};
use fantazia_core::arith::Abs;
use fantazia_core::{
    arith::{AbsAssign, Sign},
    pitch::{GeneralPitch, SpecificPitch},
};
use fantazia_internal_macro::gen_ointerval_consts;
#[cfg(feature = "macro")]
use proc_macro2::TokenStream;
#[cfg(feature = "macro")]
use quote::{ToTokens, quote};
use std::{
    fmt::{self, Debug, Display, Write},
    iter::Sum,
    num::ParseIntError,
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
    str::FromStr,
};
use thiserror::Error;

use crate::{
    acci::Acci,
    pitch::{OPitch, Pitch},
    step::{OStep, Step},
};

// Mark: Interval Quality

/// **Interval quality**, including **perfect**, **major**, **minor**, **augmented**, **diminished**,
/// and **multiply-augmented** / **diminished**.
///
/// Internally, the interval quality is represented by an integer value:
///
/// | Value   | Quality                          |
/// |---------|----------------------------------|
/// | `..=-3` | doubly / triply / ... diminished |
/// | `-2`    | diminished                       |
/// | `-1`    | minor                            |
/// | `0`     | perfect                          |
/// | `1`     | major                            |
/// | `2`     | augmented                        |
/// | `3..`   | doubly / triply / ... augmented  |
#[derive(Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq, Neg, From, Into)]
#[const_impl]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IntervalQual(i8);

impl IntervalQual {
    /// doubly augmented
    pub const DOUBLY_AUGMENTED: Self = Self(3);
    /// augmented
    pub const AUGMENTED: Self = Self(2);
    /// major (second, third, sixth, seventh only)
    pub const MAJOR: Self = Self(1);
    /// perfect (unison, fourth, fifth only)
    pub const PERFECT: Self = Self(0);
    /// minor (second, third, sixth, seventh only)
    pub const MINOR: Self = Self(-1);
    /// diminished
    pub const DIMINISHED: Self = Self(-2);
    /// doubly diminished
    pub const DOUBLY_DIMINISHED: Self = Self(-3);

    /// Creates a (multiply-)augmented interval quality
    pub const fn augmented(n: u8) -> Self {
        Self(1 + n as i8)
    }

    /// Creates a (multiply-)diminished interval quality
    pub const fn diminished(n: u8) -> Self {
        Self(-1 - n as i8)
    }

    pub const fn as_i8(&self) -> i8 {
        self.0
    }
}

impl OPitch {
    /// Gents the **interval quality** corresponding to the pitch when regarding the pitch as an
    /// interval from C.
    pub const fn qual(&self) -> IntervalQual {
        use OStep::*;
        match (self.step, self.acci().0) {
            (C | F | G, n) => match n {
                1.. => IntervalQual(n + 1),
                0 => IntervalQual::PERFECT,
                ..=-1 => IntervalQual(n - 1),
            },
            (_, n) => match n {
                1.. => IntervalQual(n + 1),
                0 => IntervalQual::MAJOR,
                ..=-1 => IntervalQual(n),
            },
        }
    }
}

impl Pitch {
    /// Gets the **interval quality** corresponding to the pitch when regarding the pitch as an
    /// interval from C.
    pub const fn qual(&self) -> IntervalQual {
        self.generalize().qual()
    }

    pub const fn abs_qual(&self) -> IntervalQual {
        if self.sign().is_lt() {
            -self.qual()
        } else {
            self.qual()
        }
    }
}

const impl PartialOrd for IntervalQual {
    /// Compare two interval qualities. Note that **major** / **minor** quality cannot co-exist with
    /// **perfect** quality in the same step, so the comparison result between them is [`None`].
    /// (And that's why [`IntervalQual`] is not [`Ord`].)
    ///
    /// ```
    /// use std::cmp::Ordering::*;
    /// use fantazia_edo12::interval::IntervalQual;
    ///
    /// assert_eq!(IntervalQual::MAJOR.partial_cmp(&IntervalQual::MINOR), Some(Greater));
    /// assert_eq!(IntervalQual::MAJOR.partial_cmp(&IntervalQual::MAJOR), Some(Equal));
    /// assert_eq!(IntervalQual::MAJOR.partial_cmp(&IntervalQual::AUGMENTED), Some(Less));
    /// assert_eq!(IntervalQual::MINOR.partial_cmp(&IntervalQual::DIMINISHED), Some(Greater));
    /// assert_eq!(IntervalQual::PERFECT.partial_cmp(&IntervalQual::AUGMENTED), Some(Less));
    /// assert_eq!(IntervalQual::PERFECT.partial_cmp(&IntervalQual::DIMINISHED), Some(Greater));
    /// assert_eq!(IntervalQual::PERFECT.partial_cmp(&IntervalQual::MAJOR), None);
    /// assert_eq!(IntervalQual::PERFECT.partial_cmp(&IntervalQual::MINOR), None);
    /// ```
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self.0.abs(), other.0.abs()) {
            (1, 0) | (0, 1) => None,
            _ => Some(self.0.cmp(&other.0)),
        }
    }
}

const impl Sign for IntervalQual {
    fn sign(&self) -> std::cmp::Ordering {
        self.0.cmp(&0)
    }
}

const impl Abs for IntervalQual {
    type Output = Self;
    fn abs(self) -> Self::Output {
        Self(self.0.abs())
    }
}

const impl AbsAssign for IntervalQual {
    fn abs_assign(&mut self) {
        *self = self.abs()
    }
}

impl Debug for IntervalQual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            n @ 2.. => write!(f, "Augmented({})", n - 1)?,
            1 => f.write_str("Major")?,
            0 => f.write_str("Perfect")?,
            -1 => f.write_str("Minor")?,
            n @ ..=-2 => write!(f, "Diminished({})", -n - 1)?,
        }
        Ok(())
    }
}

impl Display for IntervalQual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            n @ 5.. => write!(f, "{}A", n - 1)?,
            n @ 2..=4 => {
                for _ in 0..(n - 1) {
                    f.write_char('A')?;
                }
            }
            1 => f.write_char('M')?,
            0 => f.write_char('P')?,
            -1 => f.write_char('m')?,
            n @ -4..=-2 => {
                for _ in 0..(-n - 1) {
                    f.write_char('d')?
                }
            }
            n @ ..=-5 => write!(f, "{}d", -n - 1)?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseIntervalQualError {
    #[error("invalid multiple number - {0}")]
    InvalidMultipleNum(#[from] ParseIntError),
    #[error("{0:?} is not a valid quality")]
    InvalidQual(String),
}

impl FromStr for IntervalQual {
    type Err = ParseIntervalQualError;
    /// Parses an interval quality from string.
    /// A valid input should be matched by the regex `P|M|m|A+|d+|\d+(?:A|d)`
    ///
    /// ```
    /// use fantazia_edo12::interval::IntervalQual;
    ///
    /// assert_eq!("M".parse().ok(), Some(IntervalQual::MAJOR));
    /// assert_eq!("m".parse().ok(), Some(IntervalQual::MINOR));
    /// assert_eq!("P".parse().ok(), Some(IntervalQual::PERFECT));
    /// assert_eq!("A".parse().ok(), Some(IntervalQual::AUGMENTED));
    /// assert_eq!("AA".parse().ok(), Some(IntervalQual::DOUBLY_AUGMENTED));
    /// assert_eq!("d".parse().ok(), Some(IntervalQual::DIMINISHED));
    /// assert_eq!("dd".parse().ok(), Some(IntervalQual::DOUBLY_DIMINISHED));
    /// assert_eq!("AAA".parse().ok(), Some(IntervalQual::augmented(3)));
    /// assert_eq!("3A".parse().ok(), Some(IntervalQual::augmented(3)));
    /// assert_eq!("ddd".parse().ok(), Some(IntervalQual::diminished(3)));
    /// assert_eq!("3d".parse().ok(), Some(IntervalQual::diminished(3)));
    /// assert_eq!("x".parse::<IntervalQual>().ok(), None);
    /// assert_eq!("4AAx".parse::<IntervalQual>().ok(), None);
    /// assert_eq!("4ddx".parse::<IntervalQual>().ok(), None);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid_qual = |s: &str| ParseIntervalQualError::InvalidQual(s.to_string());
        if let Some(idx) = s.find(|ch: char| ch.is_ascii_alphabetic()) {
            let (multiple_num_src, letter_src) = s.split_at(idx);
            if multiple_num_src.is_empty() {
                match letter_src {
                    "M" => Ok(Self::MAJOR),
                    "m" => Ok(Self::MINOR),
                    "P" => Ok(Self::PERFECT),
                    s => {
                        let mut iter = s.chars().peekable();
                        let mut n = 0u8;
                        match iter.peek() {
                            Some('A') => {
                                for ch in iter {
                                    if ch == 'A' {
                                        n += 1;
                                    } else {
                                        return Err(invalid_qual(s));
                                    }
                                }
                                Ok(Self::augmented(n))
                            }
                            Some('d') => {
                                for ch in iter {
                                    if ch == 'd' {
                                        n += 1;
                                    } else {
                                        return Err(invalid_qual(s));
                                    }
                                }
                                Ok(Self::diminished(n))
                            }
                            _ => Err(invalid_qual(s)),
                        }
                    }
                }
            } else {
                let n: u8 = multiple_num_src.parse()?;
                match letter_src {
                    "A" => Ok(Self::augmented(n)),
                    "d" => Ok(Self::diminished(n)),
                    s => Err(invalid_qual(s)),
                }
            }
        } else {
            Err(invalid_qual(s))
        }
    }
}

// MARK: Octave Interval

#[derive(Debug, Copy)]
#[derive_const(Clone, PartialEq, Eq, PartialOrd)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OInterval {
    step: OStep,
    qual: IntervalQual,
}

#[derive(Error, Debug, Clone)]
#[error("interval quality {qual:?} is invalid for step {step}")]
pub struct InvalidOInterval {
    pub step: OStep,
    pub qual: IntervalQual,
}

gen_ointerval_consts!();

impl OInterval {
    /// Creates a new interval, checking if the combination of step and quality is valid.
    pub const fn new(
        step: impl [const] Into<OStep>,
        qual: impl [const] Into<IntervalQual>,
    ) -> Result<Self, InvalidOInterval> {
        use OStep::*;
        let step = step.into();
        let qual = qual.into();
        match (step, qual.0.abs()) {
            // intervals that does not exist:
            // major / minor - unison / fourth / fifth
            // perfect - second / third / sixth / seventh
            (C | F | G, 1) | (D | E | A | B, 0) => Err(InvalidOInterval { step, qual }),
            _ => Ok(Self { step, qual }),
        }
    }

    /// Creates a new interval without checking whether the quality is valid for the step.
    ///
    /// # Safety
    ///
    /// Caller to this function must make sure that the interval quality is valid for this step.
    /// Otherwise, undefined behavior may happen when creating pitch from the interval or comparing
    /// different intervals.
    #[inline(always)]
    pub const unsafe fn new_unchecked(
        step: impl [const] Into<OStep>,
        qual: impl [const] Into<IntervalQual>,
    ) -> Self {
        Self {
            step: step.into(),
            qual: qual.into(),
        }
    }

    pub const fn step(&self) -> OStep {
        self.step
    }

    pub const fn qual(&self) -> IntervalQual {
        self.qual
    }

    pub const fn pitch(&self) -> OPitch {
        (*self).into()
    }
}

impl OPitch {
    pub const fn interval(&self) -> OInterval {
        (*self).into()
    }
}

const impl From<OPitch> for OInterval {
    fn from(value: OPitch) -> Self {
        Self {
            step: value.step,
            qual: value.qual(),
        }
    }
}

const impl From<OInterval> for OPitch {
    fn from(value: OInterval) -> Self {
        use OStep::*;
        let acci = Acci(match (value.step, value.qual.0) {
            (C | F | G, n) => match n {
                2.. => n - 1,
                0 => 0,
                ..=-2 => n + 1,
                _ => unreachable!(),
            },
            (_, n) => match n {
                2.. => n - 1,
                1 => 0,
                ..=-1 => n,
                _ => unreachable!(),
            },
        });
        Self::diatonic(value.step).with_acci(acci)
    }
}

const impl GeneralPitch<Interval> for OInterval {
    fn put_in_octave(self, octave: i8) -> Interval {
        Interval {
            step: self.step.put_in_octave(octave),
            qual: self.qual,
        }
    }
}

const impl Default for OInterval {
    fn default() -> Self {
        Self::PERFECT_UNISON
    }
}

#[allow(clippy::derive_ord_xor_partial_ord)]
const impl Ord for OInterval {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let Some(result) = self.partial_cmp(other) else {
            // Comparison failed.
            // This is likely because an interval with invalid quality has been created,
            // which cannot happen in safe code.
            unreachable!();
        };
        result
    }
}

const impl Add for OInterval {
    type Output = Self;
    fn add(self, rhs: OInterval) -> Self::Output {
        (OPitch::from(self) + OPitch::from(rhs)).into()
    }
}

const impl AddAssign for OInterval {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

const impl Add<OInterval> for OPitch {
    type Output = Self;
    fn add(self, rhs: OInterval) -> Self::Output {
        self + Self::from(rhs)
    }
}

const impl AddAssign<OInterval> for OPitch {
    fn add_assign(&mut self, rhs: OInterval) {
        *self += Self::from(rhs);
    }
}

impl Sum for OInterval {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.map(OPitch::from).sum::<OPitch>().into()
    }
}

const impl Sub for OInterval {
    type Output = Self;
    fn sub(self, rhs: OInterval) -> Self::Output {
        (OPitch::from(self) - OPitch::from(rhs)).into()
    }
}

const impl SubAssign for OInterval {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

const impl Sub<OInterval> for OPitch {
    type Output = Self;
    fn sub(self, rhs: OInterval) -> Self::Output {
        self - Self::from(rhs)
    }
}

const impl SubAssign<OInterval> for OPitch {
    fn sub_assign(&mut self, rhs: OInterval) {
        *self -= Self::from(rhs);
    }
}

const impl Mul<i8> for OInterval {
    type Output = Self;
    fn mul(self, rhs: i8) -> Self::Output {
        (OPitch::from(self) * rhs).into()
    }
}

const impl Mul<OInterval> for i8 {
    type Output = OInterval;
    fn mul(self, rhs: OInterval) -> Self::Output {
        rhs * self
    }
}

const impl Neg for OInterval {
    type Output = Self;
    fn neg(self) -> Self::Output {
        (-OPitch::from(self)).into()
    }
}

impl Display for OInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.qual, self.step as i8 + 1)
    }
}

#[derive(Debug, Copy)]
#[derive_const(Clone, PartialEq, Eq, PartialOrd)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interval {
    step: Step,
    qual: IntervalQual,
}

const impl SpecificPitch<OInterval> for Interval {
    fn generalize(self) -> OInterval {
        OInterval {
            step: self.step.generalize(),
            qual: self.qual,
        }
    }

    fn octave(&self) -> i8 {
        self.step.octave()
    }
}

impl Interval {
    /// Creates a new interval without checking whether the quality is valid for the step.
    ///
    /// # Safety
    ///
    /// Caller to this function must make sure that the interval quality is valid for this step.
    /// Otherwise, undefined behavior may happen when creating pitch from the interval or comparing
    /// different intervals.
    pub const unsafe fn new_unchecked(
        step: impl [const] Into<Step>,
        qual: impl [const] Into<IntervalQual>,
    ) -> Self {
        Self {
            step: step.into(),
            qual: qual.into(),
        }
    }
    pub const fn step(&self) -> Step {
        self.step
    }
    pub const fn qual(&self) -> IntervalQual {
        self.qual
    }
}

#[allow(clippy::derive_ord_xor_partial_ord)]
const impl Ord for Interval {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let Some(result) = self.partial_cmp(other) else {
            unreachable!()
        };
        result
    }
}

const impl From<Pitch> for Interval {
    fn from(value: Pitch) -> Self {
        Self {
            step: value.step,
            qual: value.qual(),
        }
    }
}

const impl From<Interval> for Pitch {
    fn from(value: Interval) -> Self {
        let octave = value.octave();
        value.generalize().pitch().put_in_octave(octave)
    }
}

const impl Add for Interval {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        (Pitch::from(self) + Pitch::from(rhs)).into()
    }
}

const impl AddAssign for Interval {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

const impl Sub for Interval {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        (Pitch::from(self) - Pitch::from(rhs)).into()
    }
}

const impl SubAssign for Interval {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

const impl Mul<i8> for Interval {
    type Output = Self;
    fn mul(self, rhs: i8) -> Self::Output {
        (Pitch::from(self) * rhs).into()
    }
}

const impl Mul<Interval> for i8 {
    type Output = Interval;
    fn mul(self, rhs: Interval) -> Self::Output {
        rhs * self
    }
}

const impl Neg for Interval {
    type Output = Self;
    fn neg(self) -> Self::Output {
        (-Pitch::from(self)).into()
    }
}

const impl Abs for Interval {
    type Output = Self;
    fn abs(self) -> Self::Output {
        Self {
            step: self.step.abs(),
            qual: self.qual.abs(),
        }
    }
}

const impl AbsAssign for Interval {
    fn abs_assign(&mut self) {
        *self = self.abs();
    }
}

const impl Sign for Interval {
    fn sign(&self) -> std::cmp::Ordering {
        self.step.sign().then_with(const || self.qual.sign())
    }
}

fn fmt_interval(
    interval: Interval,
    f: &mut fmt::Formatter<'_>,
    plus_sign: bool,
    use_octave: bool,
) -> fmt::Result {
    if plus_sign {
        f.write_char('+')?;
    }
    if use_octave {
        write!(f, "{}_{}", interval.generalize(), interval.octave())?;
    } else {
        write!(f, "{}", interval.qual())?;
        if interval.step.0 == i8::MAX {
            f.write_str("128")?;
        } else if interval.step.0 == i8::MIN {
            f.write_str("129")?;
        } else {
            write!(f, "{}", interval.step.0 + 1)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Error)]
pub enum ParseIntervalError {
    #[error(transparent)]
    ParseOIntervalError(#[from] ParseOIntervalError),
    #[error("invalid octave - {0}")]
    ParseOctaveError(ParseIntError),
    #[error("missing interval step")]
    MissingStep,
    #[error("invalid step - {0}")]
    InvalidStep(ParseIntError),
    #[error(transparent)]
    InvalidQual(#[from] ParseIntervalQualError),
}

impl Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let use_octave = f.options().get_alternate();
        let (plus_sign, use_negative) = match f.options().get_sign() {
            Some(fmt::Sign::Plus) => (true, true),
            Some(fmt::Sign::Minus) => (false, true),
            _ => (false, !use_octave),
        };
        if self.sign().is_lt() && use_negative {
            f.write_char('-')?;
            fmt_interval(-*self, f, false, use_octave)?;
        } else {
            fmt_interval(*self, f, plus_sign && self.sign().is_gt(), use_octave)?;
        }
        Ok(())
    }
}

impl FromStr for Interval {
    type Err = ParseIntervalError;
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let sign = match s.chars().next() {
            Some('+') => {
                s = &s[1..];
                true
            }
            Some('-') => {
                s = &s[1..];
                false
            }
            _ => true,
        };
        if let Some((ointerval_src, octave_src)) = s.rsplit_once('_') {
            let ointerval = ointerval_src.parse::<OInterval>()?;
            let octave = octave_src
                .parse::<i8>()
                .map_err(ParseIntervalError::ParseOctaveError)?;
            let mut interval = ointerval.put_in_octave(octave);
            if !sign {
                interval = -interval;
            }
            Ok(interval)
        } else {
            let Some(step_start_idx) = s.rfind(|ch: char| !ch.is_ascii_digit()).map(|v| v + 1)
            else {
                return Err(ParseIntervalError::MissingStep);
            };
            let step = match &s[step_start_idx..] {
                "128" => i8::MAX,
                s => s.parse::<i8>().map_err(ParseIntervalError::InvalidStep)? - 1,
            };
            let step = Step(step);
            let qual = s[..step_start_idx].parse::<IntervalQual>()?;
            let mut interval = Self { step, qual };
            if !sign {
                interval = -interval;
            }
            Ok(interval)
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseOIntervalError {
    #[error("invalid step number - {0}")]
    InvalidStep(#[from] ParseIntError),
    #[error(transparent)]
    InvalidQual(#[from] ParseIntervalQualError),
    #[error("missing interval step")]
    MissingStep,
    #[error("step number {0} out of range `1..=7`")]
    StepOutOfRange(i8),
    #[error(transparent)]
    InvalidOInterval(#[from] InvalidOInterval),
}

fn parse_ointerval(mut src: &str) -> Result<(bool, OStep, IntervalQual), ParseOIntervalError> {
    let sign = match src.chars().next() {
        Some('+') => {
            src = &src[1..];
            true
        }
        Some('-') => {
            src = &src[1..];
            false
        }
        _ => true,
    };
    let Some(step_start_idx) = src.rfind(|ch: char| !ch.is_ascii_digit()).map(|v| v + 1) else {
        return Err(ParseOIntervalError::MissingStep);
    };
    let qual = src[..step_start_idx].parse()?;
    let step: i8 = src[step_start_idx..].parse()?;
    let step = if (1..=7).contains(&step) {
        unsafe { OStep::new_unchecked(step - 1) }
    } else {
        return Err(ParseOIntervalError::StepOutOfRange(step));
    };
    Ok((sign, step, qual))
}

impl FromStr for OInterval {
    type Err = ParseOIntervalError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (sign, step, qual) = parse_ointerval(s)?;
        let mut result = Self::new(step, qual)?;
        if !sign {
            result = -result
        }
        Ok(result)
    }
}

#[cfg(feature = "macro")]
impl ToTokens for IntervalQual {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = self.0;
        tokens.extend(quote! {
            ::fantazia::edo12::interval::IntervalQual::from(#value)
        });
    }
}

#[cfg(feature = "macro")]
impl ToTokens for OInterval {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let OInterval { step, qual } = self;
        tokens.extend(quote! {
            {
                // SAFETY: macro-generated code that converts a valid interval to tokens
                unsafe {
                    ::fantazia::edo12::interval::OInterval::new_unchecked(#step, #qual)
                }
            }
        });
    }
}

#[cfg(feature = "macro")]
impl ToTokens for Interval {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Interval { step, qual } = self;
        tokens.extend(quote! {
            {
                // SAFETY: macro-generated code that converts a valid interval to tokens
                unsafe {
                    ::fantazia::edo12::interval::Interval::new_unchecked(#step, #qual)
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fantazia_core::pitch::{GeneralPitch, SpecificPitch};

    #[test]
    fn test_const_gen() {
        let _ointerval_consts = vec![
            OInterval::PERFECT_UNISON,
            OInterval::MINOR_SECOND,
            OInterval::MAJOR_SECOND,
        ];
    }

    #[test]
    fn test_interval_fmt() {
        let major_third = OInterval::MAJOR_THIRD.put_in_octave(0);
        let major_tenth = major_third.with_octave(1);
        let major_third_down = -major_third;
        let major_tenth_down = -major_tenth;
        let perfect_unison = OInterval::PERFECT_UNISON.put_in_octave(0);

        assert_eq!(format!("{}", major_third), "M3");
        assert_eq!(format!("{:+}", major_third), "+M3");
        assert_eq!(format!("{:-}", major_third), "M3");
        assert_eq!(format!("{:#}", major_third), "M3_0");
        assert_eq!(format!("{:+#}", major_third), "+M3_0");
        assert_eq!(format!("{:-#}", major_third), "M3_0");

        assert_eq!(format!("{}", major_tenth), "M10");
        assert_eq!(format!("{:+}", major_tenth), "+M10");
        assert_eq!(format!("{:-}", major_tenth), "M10");
        assert_eq!(format!("{:#}", major_tenth), "M3_1");
        assert_eq!(format!("{:+#}", major_tenth), "+M3_1");
        assert_eq!(format!("{:-#}", major_tenth), "M3_1");

        assert_eq!(format!("{}", major_third_down), "-M3");
        assert_eq!(format!("{:+}", major_third_down), "-M3");
        assert_eq!(format!("{:-}", major_third_down), "-M3");
        assert_eq!(format!("{:#}", major_third_down), "m6_-1");
        assert_eq!(format!("{:+#}", major_third_down), "-M3_0");
        assert_eq!(format!("{:-#}", major_third_down), "-M3_0");

        assert_eq!(format!("{}", major_tenth_down), "-M10");
        assert_eq!(format!("{:+}", major_tenth_down), "-M10");
        assert_eq!(format!("{:-}", major_tenth_down), "-M10");
        assert_eq!(format!("{:#}", major_tenth_down), "m6_-2");
        assert_eq!(format!("{:+#}", major_tenth_down), "-M3_1");
        assert_eq!(format!("{:-#}", major_tenth_down), "-M3_1");

        assert_eq!(format!("{}", perfect_unison), "P1");
        assert_eq!(format!("{:+}", perfect_unison), "P1");
        assert_eq!(format!("{:-}", perfect_unison), "P1");
        assert_eq!(format!("{:#}", perfect_unison), "P1_0");
        assert_eq!(format!("{:+#}", perfect_unison), "P1_0");
        assert_eq!(format!("{:-#}", perfect_unison), "P1_0");
    }

    #[test]
    fn test_interval_parse() {
        let major_third = OInterval::MAJOR_THIRD.put_in_octave(0);
        let major_tenth = major_third.with_octave(1);
        let major_third_down = -major_third;
        let major_tenth_down = -major_tenth;

        assert_eq!("M3".parse().ok(), Some(major_third));
        assert_eq!("+M3".parse().ok(), Some(major_third));
        assert_eq!("-M3".parse().ok(), Some(major_third_down));
        assert_eq!("m6_-1".parse().ok(), Some(major_third_down));
        assert_eq!("-m6_-1".parse().ok(), Some(major_third));
        assert_eq!("M-3".parse::<Interval>().ok(), None);
        assert_eq!("+M-3".parse::<Interval>().ok(), None);
        assert_eq!("-M-3".parse::<Interval>().ok(), None);

        assert_eq!("M10".parse().ok(), Some(major_tenth));
        assert_eq!("+M10".parse().ok(), Some(major_tenth));
        assert_eq!("M3_1".parse().ok(), Some(major_tenth));
        assert_eq!("+M3_1".parse().ok(), Some(major_tenth));
        assert_eq!("-M10".parse().ok(), Some(major_tenth_down));
        assert_eq!("-M3_1".parse().ok(), Some(major_tenth_down));
        assert_eq!("m6_-2".parse().ok(), Some(major_tenth_down));
        assert_eq!("M-10".parse::<Interval>().ok(), None);
        assert_eq!("+M-10".parse::<Interval>().ok(), None);
        assert_eq!("-M-10".parse::<Interval>().ok(), None);
    }
}
