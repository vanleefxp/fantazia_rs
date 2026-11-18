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

    /// Returns the internal integer representation of the quality.
    ///
    /// See the [type-level documentation](IntervalQual) for the meaning of each value.
    pub const fn as_i8(&self) -> i8 {
        self.0
    }
}

impl OPitch {
    /// Gets the **interval quality** corresponding to the pitch when regarding the pitch as an
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

    /// Gets the **absolute interval quality**, i.e. the quality of the interval without its
    /// direction.
    ///
    /// This is the quality of [`Abs::abs`] of the pitch.
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

/// Error type returned when parsing an [`IntervalQual`].
#[derive(Debug, Clone, Error)]
pub enum ParseIntervalQualError {
    /// The multiple number of a `3A` / `3d` style quality is not a valid integer.
    #[error("invalid multiple number - {0}")]
    InvalidMultipleNum(#[from] ParseIntError),
    /// The input does not name a valid interval quality.
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
/// An octave-unspecific interval, i.e. an interval that does not cross more than one octave.
///
/// An `OInterval` is described by the [`OStep`] of its upper note and its [`IntervalQual`], so e.g.
/// a major third is `OStep::E` with [`IntervalQual::MAJOR`]. Only valid combinations of step and
/// quality can be created through [`OInterval::new`].
///
/// Conversions to and from [`OPitch`] treat the pitch as an interval from `C`.
pub struct OInterval {
    step: OStep,
    qual: IntervalQual,
}

/// Error type returned when the combination of a step and a quality does not describe a valid
/// interval.
///
/// For example a *major unison* or a *perfect third* do not exist.
#[derive(Error, Debug, Clone)]
#[error("interval quality {qual:?} is invalid for step {step}")]
pub struct InvalidOInterval {
    /// The step of the interval.
    pub step: OStep,
    /// The invalid quality for that step.
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

    /// Returns the step of the interval, i.e. the note letter of its upper note.
    pub const fn step(&self) -> OStep {
        self.step
    }

    /// Returns the quality of the interval.
    pub const fn qual(&self) -> IntervalQual {
        self.qual
    }

    /// Converts the interval to the [`OPitch`] it spans from `C`.
    ///
    /// ```
    /// use fantazia_edo12::{interval::OInterval, pitch::OPitch};
    ///
    /// assert_eq!(OInterval::MAJOR_THIRD.pitch(), OPitch::E);
    /// assert_eq!(OInterval::PERFECT_FIFTH.pitch(), OPitch::G);
    /// ```
    pub const fn pitch(&self) -> OPitch {
        (*self).into()
    }
}

impl OPitch {
    /// Interprets the pitch as an interval from `C`.
    ///
    /// ```
    /// use fantazia_edo12::{interval::OInterval, pitch::OPitch};
    ///
    /// assert_eq!(OPitch::E.interval(), OInterval::MAJOR_THIRD);
    /// assert_eq!(OPitch::G.interval(), OInterval::PERFECT_FIFTH);
    /// ```
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

/// Formats the interval as its quality followed by its number, e.g. `M3` or `d5`.
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
/// An octave-specific interval, i.e. an interval that may cross one or more octaves.
///
/// This is the octave-specific counterpart of [`OInterval`]: the [`Step`] keeps the octave of the
/// interval, so a major tenth is `Step::E_1` with [`IntervalQual::MAJOR`]. Conversions to and from
/// [`Pitch`] treat the pitch as an interval from `C`.
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
    /// Returns the octave-specific step of the interval, i.e. the note letter and octave of its
    /// upper note.
    pub const fn step(&self) -> Step {
        self.step
    }

    /// Returns the quality of the interval.
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

/// Error type returned when parsing an [`Interval`].
#[derive(Debug, Clone, Error)]
pub enum ParseIntervalError {
    /// The octave-unspecific part of the interval is invalid.
    #[error(transparent)]
    ParseOIntervalError(#[from] ParseOIntervalError),
    /// The octave number is not a valid integer.
    #[error("invalid octave - {0}")]
    ParseOctaveError(ParseIntError),
    /// The interval number is missing.
    #[error("missing interval step")]
    MissingStep,
    /// The interval number is not a valid integer.
    #[error("invalid step - {0}")]
    InvalidStep(ParseIntError),
    /// The interval quality is invalid.
    #[error(transparent)]
    InvalidQual(#[from] ParseIntervalQualError),
    /// The combination of step and quality does not describe a valid interval.
    #[error(transparent)]
    InvalidOInterval(#[from] InvalidOInterval),
}

/// Formats the interval with its quality followed by its number, e.g. `M3` or `M10`.
///
/// The following formatting options are supported:
///
/// - `{}` prints the interval number, e.g. `M3`, and a leading `-` for descending intervals.
/// - `{:+}` always prints a leading sign, e.g. `+M3` or `-M3`.
/// - `{:-}` omits the leading `+` sign.
/// - `{:#}` prints the octave-unspecific form and the octave instead of the interval number, e.g.
///   `M3_0` or `m6_-1`.
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

    /// Parses an [`Interval`] from an interval quality, an interval number and an optional sign.
    ///
    /// Two notations are supported:
    ///
    /// - a compound interval number, e.g. `M10`, which may be preceded by `+` or `-`;
    /// - the octave-unspecific form followed by `_` and an octave number, e.g. `M3_1`.
    ///
    /// ```
    /// use fantazia_edo12::interval::{Interval, OInterval};
    /// use fantazia_core::pitch::{GeneralPitch, SpecificPitch};
    ///
    /// let major_third = OInterval::MAJOR_THIRD.put_in_octave(0);
    /// assert_eq!("M3".parse::<Interval>().ok(), Some(major_third));
    /// assert_eq!("-M3".parse::<Interval>().ok(), Some(-major_third));
    /// assert_eq!("M3_1".parse::<Interval>().ok(), Some(major_third.with_octave(1)));
    /// assert_eq!("M-3".parse::<Interval>().ok(), None);
    /// ```
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
            // Validate the spelling against the generalized step, so that e.g. `M1` or `P3` are
            // rejected while compound intervals such as `M10` are accepted.
            OInterval::new(step.generalize(), qual)?;
            let mut interval = Self { step, qual };
            if !sign {
                interval = -interval;
            }
            Ok(interval)
        }
    }
}

/// Error type returned when parsing an [`OInterval`].
#[derive(Debug, Clone, Error)]
pub enum ParseOIntervalError {
    /// The interval number is not a valid integer.
    #[error("invalid step number - {0}")]
    InvalidStep(#[from] ParseIntError),
    /// The interval quality is invalid.
    #[error(transparent)]
    InvalidQual(#[from] ParseIntervalQualError),
    /// The interval number is missing.
    #[error("missing interval step")]
    MissingStep,
    /// The interval number is outside `1..=7`, so the interval is not octave-unspecific.
    #[error("step number {0} out of range `1..=7`")]
    StepOutOfRange(i8),
    /// The combination of step and quality is invalid.
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
    use std::cmp::Ordering;

    use fantazia_core::{
        arith::{Abs, AbsAssign, Sign},
        pitch::{GeneralPitch, SpecificPitch},
    };

    use super::*;
    use crate::acci::Acci;
    use crate::pitch::AcciOps;

    const OPITCHES: [OPitch; 17] = [
        OPitch::C,
        OPitch::C_SHARP,
        OPitch::D_FLAT,
        OPitch::D,
        OPitch::D_SHARP,
        OPitch::E_FLAT,
        OPitch::E,
        OPitch::F,
        OPitch::F_SHARP,
        OPitch::G_FLAT,
        OPitch::G,
        OPitch::G_SHARP,
        OPitch::A_FLAT,
        OPitch::A,
        OPitch::A_SHARP,
        OPitch::B_FLAT,
        OPitch::B,
    ];

    #[test]
    fn test_const_gen() {
        // Perfect-class steps only admit diminished, perfect and augmented qualities, while
        // major-class steps admit diminished, minor, major and augmented qualities.
        let expected = [
            (OInterval::DIMINISHED_UNISON, OStep::C, -2),
            (OInterval::PERFECT_UNISON, OStep::C, 0),
            (OInterval::AUGMENTED_UNISON, OStep::C, 2),
            (OInterval::DIMINISHED_SECOND, OStep::D, -2),
            (OInterval::MINOR_SECOND, OStep::D, -1),
            (OInterval::MAJOR_SECOND, OStep::D, 1),
            (OInterval::AUGMENTED_SECOND, OStep::D, 2),
            (OInterval::DIMINISHED_THIRD, OStep::E, -2),
            (OInterval::MINOR_THIRD, OStep::E, -1),
            (OInterval::MAJOR_THIRD, OStep::E, 1),
            (OInterval::AUGMENTED_THIRD, OStep::E, 2),
            (OInterval::DIMINISHED_FOURTH, OStep::F, -2),
            (OInterval::PERFECT_FOURTH, OStep::F, 0),
            (OInterval::AUGMENTED_FOURTH, OStep::F, 2),
            (OInterval::DIMINISHED_FIFTH, OStep::G, -2),
            (OInterval::PERFECT_FIFTH, OStep::G, 0),
            (OInterval::AUGMENTED_FIFTH, OStep::G, 2),
            (OInterval::DIMINISHED_SIXTH, OStep::A, -2),
            (OInterval::MINOR_SIXTH, OStep::A, -1),
            (OInterval::MAJOR_SIXTH, OStep::A, 1),
            (OInterval::AUGMENTED_SIXTH, OStep::A, 2),
            (OInterval::DIMINISHED_SEVENTH, OStep::B, -2),
            (OInterval::MINOR_SEVENTH, OStep::B, -1),
            (OInterval::MAJOR_SEVENTH, OStep::B, 1),
            (OInterval::AUGMENTED_SEVENTH, OStep::B, 2),
        ];
        assert_eq!(expected.len(), 25);
        for (interval, step, qual) in expected {
            assert_eq!(interval.step(), step, "{interval}");
            assert_eq!(interval.qual(), IntervalQual::from(qual), "{interval}");
        }
        assert_eq!(OInterval::default(), OInterval::PERFECT_UNISON);
    }

    #[test]
    fn test_interval_qual_constants() {
        assert_eq!(IntervalQual::DOUBLY_AUGMENTED.as_i8(), 3);
        assert_eq!(IntervalQual::AUGMENTED.as_i8(), 2);
        assert_eq!(IntervalQual::MAJOR.as_i8(), 1);
        assert_eq!(IntervalQual::PERFECT.as_i8(), 0);
        assert_eq!(IntervalQual::MINOR.as_i8(), -1);
        assert_eq!(IntervalQual::DIMINISHED.as_i8(), -2);
        assert_eq!(IntervalQual::DOUBLY_DIMINISHED.as_i8(), -3);

        assert_eq!(IntervalQual::augmented(0), IntervalQual::MAJOR);
        assert_eq!(IntervalQual::augmented(1), IntervalQual::AUGMENTED);
        assert_eq!(IntervalQual::augmented(2), IntervalQual::DOUBLY_AUGMENTED);
        assert_eq!(IntervalQual::augmented(3).as_i8(), 4);
        assert_eq!(IntervalQual::diminished(0), IntervalQual::MINOR);
        assert_eq!(IntervalQual::diminished(1), IntervalQual::DIMINISHED);
        assert_eq!(IntervalQual::diminished(2), IntervalQual::DOUBLY_DIMINISHED);
        assert_eq!(IntervalQual::diminished(3).as_i8(), -4);

        assert_eq!(-IntervalQual::MAJOR, IntervalQual::MINOR);
        assert_eq!(-IntervalQual::PERFECT, IntervalQual::PERFECT);
        assert_eq!(-IntervalQual::AUGMENTED, IntervalQual::DIMINISHED);
        assert_eq!(-IntervalQual::MINOR, IntervalQual::MAJOR);
    }

    #[test]
    fn test_interval_qual_display_and_parse() {
        let expected = [
            (5, "4A"),
            (4, "AAA"),
            (3, "AA"),
            (2, "A"),
            (1, "M"),
            (0, "P"),
            (-1, "m"),
            (-2, "d"),
            (-3, "dd"),
            (-4, "ddd"),
            (-5, "4d"),
        ];
        for (value, src) in expected {
            let qual = IntervalQual::from(value);
            assert_eq!(qual.to_string(), src, "value {value}");
            assert_eq!(src.parse::<IntervalQual>().unwrap(), qual, "`{src}`");
        }

        // Every quality round-trips through its display representation.
        for value in -10..=10 {
            let qual = IntervalQual::from(value);
            let src = qual.to_string();
            assert_eq!(src.parse::<IntervalQual>().unwrap(), qual, "`{src}`");
        }

        assert_eq!(
            "3A".parse::<IntervalQual>().unwrap(),
            IntervalQual::augmented(3)
        );
        assert_eq!(
            "3d".parse::<IntervalQual>().unwrap(),
            IntervalQual::diminished(3)
        );

        assert!(matches!(
            "x".parse::<IntervalQual>(),
            Err(ParseIntervalQualError::InvalidQual(_))
        ));
        assert!(matches!(
            "4AAx".parse::<IntervalQual>(),
            Err(ParseIntervalQualError::InvalidQual(_))
        ));
        assert!(matches!(
            "4x".parse::<IntervalQual>(),
            Err(ParseIntervalQualError::InvalidQual(_))
        ));
        assert_eq!(
            "+3A".parse::<IntervalQual>().unwrap(),
            IntervalQual::augmented(3)
        );
        assert!(matches!(
            "-3A".parse::<IntervalQual>(),
            Err(ParseIntervalQualError::InvalidMultipleNum(_))
        ));
        assert_eq!(
            "x".parse::<IntervalQual>().unwrap_err().to_string(),
            "\"x\" is not a valid quality"
        );
    }

    #[test]
    fn test_interval_qual_debug() {
        assert_eq!(format!("{:?}", IntervalQual::MAJOR), "Major");
        assert_eq!(format!("{:?}", IntervalQual::PERFECT), "Perfect");
        assert_eq!(format!("{:?}", IntervalQual::MINOR), "Minor");
        assert_eq!(format!("{:?}", IntervalQual::AUGMENTED), "Augmented(1)");
        assert_eq!(
            format!("{:?}", IntervalQual::DOUBLY_AUGMENTED),
            "Augmented(2)"
        );
        assert_eq!(format!("{:?}", IntervalQual::DIMINISHED), "Diminished(1)");
        assert_eq!(
            format!("{:?}", IntervalQual::DOUBLY_DIMINISHED),
            "Diminished(2)"
        );
    }

    #[test]
    fn test_interval_qual_ordering() {
        assert_eq!(
            IntervalQual::MAJOR.partial_cmp(&IntervalQual::MINOR),
            Some(Ordering::Greater)
        );
        assert_eq!(
            IntervalQual::MAJOR.partial_cmp(&IntervalQual::MAJOR),
            Some(Ordering::Equal)
        );
        assert_eq!(
            IntervalQual::MAJOR.partial_cmp(&IntervalQual::AUGMENTED),
            Some(Ordering::Less)
        );
        assert_eq!(
            IntervalQual::MINOR.partial_cmp(&IntervalQual::DIMINISHED),
            Some(Ordering::Greater)
        );
        assert_eq!(
            IntervalQual::DOUBLY_AUGMENTED.partial_cmp(&IntervalQual::AUGMENTED),
            Some(Ordering::Greater)
        );
        assert_eq!(
            IntervalQual::PERFECT.partial_cmp(&IntervalQual::AUGMENTED),
            Some(Ordering::Less)
        );
        assert_eq!(
            IntervalQual::PERFECT.partial_cmp(&IntervalQual::DIMINISHED),
            Some(Ordering::Greater)
        );

        // Major / minor cannot be compared with perfect.
        assert_eq!(
            IntervalQual::PERFECT.partial_cmp(&IntervalQual::MAJOR),
            None
        );
        assert_eq!(
            IntervalQual::PERFECT.partial_cmp(&IntervalQual::MINOR),
            None
        );
        assert_eq!(
            IntervalQual::MAJOR.partial_cmp(&IntervalQual::PERFECT),
            None
        );
        assert_eq!(
            IntervalQual::MINOR.partial_cmp(&IntervalQual::PERFECT),
            None
        );
    }

    #[test]
    fn test_interval_qual_abs_and_sign() {
        assert_eq!(IntervalQual::MINOR.abs(), IntervalQual::MAJOR);
        assert_eq!(
            IntervalQual::DOUBLY_DIMINISHED.abs(),
            IntervalQual::DOUBLY_AUGMENTED
        );
        assert_eq!(IntervalQual::MINOR.sign(), Ordering::Less);
        assert_eq!(IntervalQual::PERFECT.sign(), Ordering::Equal);
        assert_eq!(IntervalQual::AUGMENTED.sign(), Ordering::Greater);

        let mut qual = IntervalQual::MINOR;
        qual.abs_assign();
        assert_eq!(qual, IntervalQual::MAJOR);
    }

    #[test]
    fn test_ointerval_new() {
        for step in [OStep::C, OStep::F, OStep::G] {
            assert!(OInterval::new(step, IntervalQual::PERFECT).is_ok());
            assert!(OInterval::new(step, IntervalQual::AUGMENTED).is_ok());
            assert!(OInterval::new(step, IntervalQual::DOUBLY_AUGMENTED).is_ok());
            assert!(OInterval::new(step, IntervalQual::DIMINISHED).is_ok());
            assert!(OInterval::new(step, IntervalQual::DOUBLY_DIMINISHED).is_ok());
            assert!(OInterval::new(step, IntervalQual::MAJOR).is_err());
            assert!(OInterval::new(step, IntervalQual::MINOR).is_err());
        }
        for step in [OStep::D, OStep::E, OStep::A, OStep::B] {
            assert!(OInterval::new(step, IntervalQual::MAJOR).is_ok());
            assert!(OInterval::new(step, IntervalQual::MINOR).is_ok());
            assert!(OInterval::new(step, IntervalQual::AUGMENTED).is_ok());
            assert!(OInterval::new(step, IntervalQual::DIMINISHED).is_ok());
            assert!(OInterval::new(step, IntervalQual::PERFECT).is_err());
        }

        let err = OInterval::new(OStep::C, IntervalQual::MAJOR).unwrap_err();
        assert_eq!(err.step, OStep::C);
        assert_eq!(err.qual, IntervalQual::MAJOR);
        assert_eq!(
            err.to_string(),
            "interval quality Major is invalid for step C"
        );

        // SAFETY: the combinations below are valid.
        let interval = unsafe { OInterval::new_unchecked(OStep::E, IntervalQual::MAJOR) };
        assert_eq!(interval, OInterval::MAJOR_THIRD);
    }

    #[test]
    fn test_ointerval_getters_and_pitch() {
        assert_eq!(OInterval::MAJOR_THIRD.step(), OStep::E);
        assert_eq!(OInterval::MAJOR_THIRD.qual(), IntervalQual::MAJOR);
        assert_eq!(OInterval::MAJOR_THIRD.pitch(), OPitch::E);
        assert_eq!(OPitch::E.interval(), OInterval::MAJOR_THIRD);
        assert_eq!(OInterval::PERFECT_FIFTH.pitch(), OPitch::G);
        assert_eq!(OPitch::G.interval(), OInterval::PERFECT_FIFTH);
    }

    #[test]
    fn test_ointerval_pitch_roundtrip() {
        for step_idx in 0..7 {
            let step = OStep::new(step_idx).unwrap();
            for qual_value in -8..=8 {
                let qual = IntervalQual::from(qual_value);
                let Ok(interval) = OInterval::new(step, qual) else {
                    continue;
                };
                let pitch = interval.pitch();
                assert_eq!(pitch.interval(), interval, "{interval}");
                assert_eq!(pitch.step, step, "{interval}");
            }
        }

        for pitch in OPITCHES {
            assert_eq!(pitch.interval().pitch(), pitch);
        }
    }

    #[test]
    fn test_ointerval_arithmetic() {
        assert_eq!(
            OInterval::MAJOR_THIRD + OInterval::MINOR_THIRD,
            OInterval::PERFECT_FIFTH
        );
        assert_eq!(
            OInterval::PERFECT_FIFTH - OInterval::MAJOR_THIRD,
            OInterval::MINOR_THIRD
        );
        assert_eq!(OInterval::PERFECT_FIFTH * 2, OInterval::MAJOR_SECOND);
        assert_eq!(2 * OInterval::MAJOR_THIRD, OInterval::AUGMENTED_FIFTH);
        let zero = 0;
        assert_eq!(OInterval::MAJOR_THIRD * zero, OInterval::PERFECT_UNISON);
        assert_eq!(-OInterval::MAJOR_THIRD, OInterval::MINOR_SIXTH);
        assert_eq!(
            OInterval::PERFECT_FIFTH + -OInterval::PERFECT_FIFTH,
            OInterval::PERFECT_UNISON
        );
        assert_eq!(
            [OInterval::MAJOR_THIRD, OInterval::MINOR_THIRD]
                .into_iter()
                .sum::<OInterval>(),
            OInterval::PERFECT_FIFTH
        );
        assert_eq!(
            std::iter::empty::<OInterval>().sum::<OInterval>(),
            OInterval::PERFECT_UNISON
        );

        let mut interval = OInterval::MAJOR_THIRD;
        interval += OInterval::MINOR_THIRD;
        assert_eq!(interval, OInterval::PERFECT_FIFTH);
        interval -= OInterval::MAJOR_THIRD;
        assert_eq!(interval, OInterval::MINOR_THIRD);
    }

    #[test]
    fn test_ointerval_display() {
        assert_eq!(OInterval::PERFECT_UNISON.to_string(), "P1");
        assert_eq!(OInterval::MINOR_SECOND.to_string(), "m2");
        assert_eq!(OInterval::MAJOR_THIRD.to_string(), "M3");
        assert_eq!(OInterval::PERFECT_FOURTH.to_string(), "P4");
        assert_eq!(OInterval::DIMINISHED_FIFTH.to_string(), "d5");
        assert_eq!(OInterval::AUGMENTED_FOURTH.to_string(), "A4");
        assert_eq!(OInterval::MINOR_SEVENTH.to_string(), "m7");
        assert_eq!(OInterval::AUGMENTED_SEVENTH.to_string(), "A7");
    }

    #[test]
    fn test_ointerval_ordering() {
        assert_eq!(
            OInterval::PERFECT_UNISON.cmp(&OInterval::PERFECT_UNISON),
            Ordering::Equal
        );
        assert!(OInterval::PERFECT_UNISON < OInterval::MAJOR_SECOND);
        assert!(OInterval::MAJOR_THIRD > OInterval::MINOR_THIRD);
        assert!(OInterval::AUGMENTED_FOURTH > OInterval::PERFECT_FOURTH);
        assert!(OInterval::DIMINISHED_FIFTH < OInterval::PERFECT_FIFTH);
        assert!(OInterval::MAJOR_SEVENTH < OInterval::AUGMENTED_SEVENTH);
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

        let perfect_octave = OInterval::PERFECT_UNISON.put_in_octave(1);
        assert_eq!(format!("{}", perfect_octave), "P8");
        assert_eq!(format!("{:#}", perfect_octave), "P1_1");

        let dim_fifth = OInterval::DIMINISHED_FIFTH.put_in_octave(0);
        let aug_fourth = OInterval::AUGMENTED_FOURTH.put_in_octave(0);
        assert_eq!(format!("{}", dim_fifth), "d5");
        assert_eq!(format!("{}", aug_fourth), "A4");
        assert_eq!(format!("{:+}", aug_fourth), "+A4");
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

        assert_eq!(
            "P8".parse().ok(),
            Some(OInterval::PERFECT_UNISON.put_in_octave(1))
        );
        assert_eq!(
            "A4".parse().ok(),
            Some(OInterval::AUGMENTED_FOURTH.put_in_octave(0))
        );
        assert_eq!(
            "-A4".parse().ok(),
            Some(-OInterval::AUGMENTED_FOURTH.put_in_octave(0))
        );
    }

    #[test]
    fn test_interval_parse_errors() {
        assert!(matches!(
            "M".parse::<Interval>(),
            Err(ParseIntervalError::InvalidStep(_))
        ));
        assert!(matches!(
            "+".parse::<Interval>(),
            Err(ParseIntervalError::MissingStep)
        ));
        assert!(matches!(
            "M_0".parse::<Interval>(),
            Err(ParseIntervalError::ParseOIntervalError(_))
        ));
        assert!(matches!(
            "M1".parse::<Interval>(),
            Err(ParseIntervalError::InvalidOInterval(_))
        ));
        assert!(matches!(
            "P3".parse::<Interval>(),
            Err(ParseIntervalError::InvalidOInterval(_))
        ));
        assert!(matches!(
            "M3_x".parse::<Interval>(),
            Err(ParseIntervalError::ParseOctaveError(_))
        ));
        assert!(matches!(
            "x3".parse::<Interval>(),
            Err(ParseIntervalError::InvalidQual(_))
        ));

        // Octave-unspecific intervals reject compound numbers.
        assert!(matches!(
            "M10".parse::<OInterval>(),
            Err(ParseOIntervalError::StepOutOfRange(10))
        ));
        assert!(matches!(
            "M3_1".parse::<OInterval>(),
            Err(ParseOIntervalError::InvalidQual(_))
        ));
    }

    #[test]
    fn test_interval_pitch_conversion() {
        for octave in -2..=2 {
            for opitch in OPITCHES {
                let pitch = opitch.put_in_octave(octave);
                let interval = Interval::from(pitch);
                assert_eq!(Pitch::from(interval), pitch, "{interval}");
            }
        }

        // A major tenth is the compound form of a major third.
        let major_tenth = OInterval::MAJOR_THIRD.put_in_octave(1);
        assert_eq!(major_tenth.step(), Step::E_1);
        assert_eq!(major_tenth.octave(), 1);
        assert_eq!(major_tenth.generalize(), OInterval::MAJOR_THIRD);
        assert_eq!(Pitch::from(major_tenth), Pitch::E_1);
        assert_eq!(Interval::from(Pitch::E_1), major_tenth);
        assert_eq!(major_tenth.to_string(), "M10");
    }

    #[test]
    fn test_interval_arithmetic() {
        let major_third = OInterval::MAJOR_THIRD.put_in_octave(0);
        let minor_third = OInterval::MINOR_THIRD.put_in_octave(0);

        assert_eq!(
            major_third + minor_third,
            OInterval::PERFECT_FIFTH.put_in_octave(0)
        );
        assert_eq!(
            major_third - minor_third,
            OInterval::AUGMENTED_UNISON.put_in_octave(0)
        );
        assert_eq!(major_third * 2, OInterval::AUGMENTED_FIFTH.put_in_octave(0));
        let zero = 0;
        assert_eq!(
            major_third * zero,
            OInterval::PERFECT_UNISON.put_in_octave(0)
        );
        assert_eq!(major_third * -1, OInterval::MINOR_SIXTH.put_in_octave(-1));
        assert_eq!(-major_third, OInterval::MINOR_SIXTH.put_in_octave(-1));

        assert_eq!(major_third.abs(), major_third);
        assert_eq!((-major_third).abs(), major_third);
        assert_eq!((-major_third).sign(), Ordering::Less);
        assert_eq!(major_third.sign(), Ordering::Greater);
        assert_eq!(
            OInterval::PERFECT_UNISON.put_in_octave(0).sign(),
            Ordering::Equal
        );

        let mut interval = major_third;
        interval += minor_third;
        assert_eq!(interval, OInterval::PERFECT_FIFTH.put_in_octave(0));
        interval -= major_third;
        assert_eq!(interval, minor_third);
    }

    #[test]
    fn test_interval_abs_assign() {
        let mut interval = -OInterval::MAJOR_THIRD.put_in_octave(0);
        interval.abs_assign();
        assert_eq!(interval, OInterval::MAJOR_THIRD.put_in_octave(0));
    }

    #[test]
    fn test_interval_qual_from_pitch() {
        assert_eq!(OPitch::C.qual(), IntervalQual::PERFECT);
        assert_eq!(OPitch::E.qual(), IntervalQual::MAJOR);
        assert_eq!(OPitch::C_SHARP.qual(), IntervalQual::AUGMENTED);
        assert_eq!(
            OPitch::C.double_sharp().qual(),
            IntervalQual::DOUBLY_AUGMENTED
        );
        assert_eq!(OPitch::D_FLAT.qual(), IntervalQual::MINOR);
        assert_eq!(OPitch::D.double_flat().qual(), IntervalQual::DIMINISHED);
        assert_eq!(OPitch::G.qual(), IntervalQual::PERFECT);
        assert_eq!(
            OPitch::G.with_acci(Acci::FLAT).qual(),
            IntervalQual::DIMINISHED
        );

        let pitch_down = Pitch::C_0 - Pitch::E_0;
        assert_eq!(pitch_down.qual(), IntervalQual::MINOR);
        assert_eq!(pitch_down.abs_qual(), IntervalQual::MAJOR);
        assert_eq!(Pitch::E_0.qual(), IntervalQual::MAJOR);
        assert_eq!(Pitch::E_0.abs_qual(), IntervalQual::MAJOR);
    }
}
