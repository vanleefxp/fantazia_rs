use std::str::FromStr;
use itertools::Itertools as _;
use uncased::AsUncased as _;

use crate::pitch::edo12::err::InvalidOInterval;

use super::base::{Acci, OPitch, OStep, Pitch, STEP_NAMES, Step};
use super::interval::{Interval, IntervalDeg, IntervalQual, OInterval, OIntervalDeg};

pub mod err {
    use std::num::ParseIntError;

    use num_enum::TryFromPrimitiveError;
    use thiserror::Error;

    use crate::pitch::edo12::{OIntervalDeg, err::InvalidOInterval};

    #[derive(Debug, Clone, Error)]
    #[error("Invalid step name: `{0}`")]
    pub struct ParseOStepError(pub(super) String);

    #[derive(Debug, Error)]
    pub enum ParseAcciError {
        #[error("Unclosed bracket in accidental string.")]
        UnclosedBracket,
        #[error("Invalid character in accidental string: `{0}`.")]
        InvalidChar(char),
        #[error(transparent)]
        InvalidNum(#[from] ParseIntError)
    }

    #[derive(Debug, Error)]
    pub enum ParseStepError {
        #[error(transparent)]
        InvalidOStep(#[from] ParseOStepError),
        #[error(transparent)]
        InvalidOctave(#[from] ParseIntError),
    }

    #[derive(Debug, Error)]
    pub enum ParseOPitchError {
        #[error(transparent)]
        InvalidOStep(#[from] ParseOStepError),
        #[error(transparent)]
        InvalidAcci(#[from] ParseAcciError),
    }

    #[derive(Debug, Error)]
    pub enum ParseOIntervalDegError {
        #[error("0 is not a valid interval degree.")]
        ZeroDegree,
        #[error(transparent)]
        OutOfBounds(#[from] TryFromPrimitiveError<OIntervalDeg>),
        #[error(transparent)]
        InvalidNumber(#[from] ParseIntError),
    }

    #[derive(Debug, Error)]
    pub enum ParseIntervalQualError {
        #[error("Empty input.")]
        EmptyInput,
        #[error("Unclosed bracket in interval quality string.")]
        UnclosedBracket,
        #[error("Invalid interval quality: `{0}`.")]
        InvalidInput(String),
        #[error(transparent)]
        InvalidNumber(#[from] ParseIntError),
    }

    #[derive(Debug, Error)]
    #[error("Interval degree is missing: `{0}`.")]
    pub struct MissingDeg(pub(super) String);

    #[derive(Debug, Error)]
    pub enum ParseOIntervalError {
        #[error(transparent)]
        MissingDeg(#[from] MissingDeg),
        #[error(transparent)]
        InvalidDeg(#[from] ParseOIntervalDegError),
        #[error(transparent)]
        InvalidQual(#[from] ParseIntervalQualError),
        #[error(transparent)]
        InvalidOInterval(#[from] InvalidOInterval),
    }

    #[derive(Debug, Error)]
    pub enum ParsePitchError {
        #[error(transparent)]
        InvalidOPitch(#[from] ParseOPitchError),
        #[error(transparent)]
        InvalidOctave(#[from] ParseIntError),
    }

    #[derive(Debug, Error)]
    pub enum ParseIntervalError {
        #[error(transparent)]
        MissingDeg(#[from] MissingDeg),
        #[error(transparent)]
        InvalidDeg(#[from] ParseIntError),
        #[error(transparent)]
        InvalidQual(#[from] ParseIntervalQualError),
        #[error(transparent)]
        InvalidOInterval(#[from] InvalidOInterval),
    }
}


impl FromStr for OStep {
    type Err = err::ParseOStepError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = s.as_uncased();
        STEP_NAMES
            .get(&key)
            .cloned()
            .ok_or_else(|| err::ParseOStepError(s.to_string()))
    }
}

impl FromStr for Step {
    type Err = err::ParseStepError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(idx) = s.find('_') {
            let ostep = (&s[..idx]).parse::<OStep>()?;
            let octave_src = &s[idx + 1..];
            let octave: i8 = octave_src.parse()?;
            Ok(Step(ostep as i8 + octave * 12))
        } else {
            Ok(s.parse::<OStep>()?.into())
        }
    }
}

impl FromStr for Acci {
    type Err = err::ParseAcciError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || s == "=" {
            Ok(Self::NATURAL)
        } else if s.starts_with('[') {
            if s.ends_with(']') {
                let s = &s[1..s.len() - 1];
                Ok(Acci(s.parse::<i8>()?))
            } else {
                Err(err::ParseAcciError::UnclosedBracket)
            }
        } else {
            let mut acci: i8 = 0;
            for ch in s.chars() {
                match ch {
                    '+' => acci += 1,
                    '-' => acci -= 1,
                    _ => return Err(err::ParseAcciError::InvalidChar(ch)),
                }
            }
            Ok(Acci(acci))
        }
    }
}

impl FromStr for OPitch {
    type Err = err::ParseOPitchError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.find(|ch: char| !ch.is_ascii_alphabetic()) {
            Some(idx) => Ok(OPitch::new((&s[..idx]).parse()?, (&s[idx..]).parse()?)),
            None => Ok(s.parse::<OStep>()?.into()),
        }
    }
}

impl FromStr for OIntervalDeg {
    type Err = err::ParseOIntervalDegError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deg_plus_1: u8 = s.parse()?;
        let deg = OIntervalDeg::try_from(
            deg_plus_1
                .checked_sub(1)
                .ok_or_else(|| err::ParseOIntervalDegError::ZeroDegree)?,
        )?;
        Ok(deg)
    }
}

impl FromStr for IntervalDeg {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(idx) = s.find('_') {
            let odeg_src = &s[..idx];
            let octave_src = &s[idx + 1..];
            let odeg: OIntervalDeg = odeg_src.parse()?;
            let octave: i8 = i8::from_str_radix(octave_src, 10)?;
            Ok(IntervalDeg::from_odeg_and_octave(odeg, octave))
        } else {
            Ok(OIntervalDeg::from_str(s)?.into())
        }
    }
}

impl FromStr for IntervalQual {
    type Err = err::ParseIntervalQualError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use IntervalQual::*;
        use err::ParseIntervalQualError::*;
        match s {
            "P" => return Ok(Perfect),
            "M" => return Ok(Major),
            "m" => return Ok(Minor),
            s => {
                let mut chars = s.chars();
                match chars.next().ok_or_else(|| EmptyInput)? {
                    '[' => {
                        if !s.ends_with("]") {
                            return Err(UnclosedBracket);
                        }
                        match chars.next().unwrap() {
                            'A' => {
                                if let Some('*') = chars.next() {
                                    let n: u8 =
                                        u8::from_str_radix(chars.dropping_back(1).as_str(), 10)?;
                                    return Ok(Augmented(n));
                                }
                            }
                            'd' => {
                                if let Some('*') = chars.next() {
                                    let n: u8 =
                                        u8::from_str_radix(chars.dropping_back(1).as_str(), 10)?;
                                    return Ok(Diminished(n));
                                }
                            }
                            _ => (),
                        }
                    }
                    'A' => {
                        let mut n = 1;
                        while let Some('A') = chars.next() {
                            n += 1;
                        }
                        match chars.next() {
                            None => return Ok(Augmented(n)),
                            _ => (),
                        }
                    }
                    'd' => {
                        let mut n = 1;
                        while let Some('d') = chars.next() {
                            n += 1;
                        }
                        match chars.next() {
                            None => return Ok(Diminished(n)),
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
        }
        Err(InvalidInput(s.to_string()))
    }
}

fn split_qual_and_deg(s: &str) -> Result<(&str, &str), err::MissingDeg> {
    s
        .char_indices()
        .rev()
        .take_while(|&(_, ch)| ch.is_ascii_digit())
        .last()
        .map(|(idx, _)| (&s[..idx], &s[idx..])).ok_or_else(|| err::MissingDeg(s.to_string()))
}

impl FromStr for OInterval {
    type Err = err::ParseOIntervalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (qual, deg) = split_qual_and_deg(s)?;
        let qual: IntervalQual = qual.parse()?;
        let deg: OIntervalDeg = deg.parse()?;
        Ok(OInterval::try_from_deg_and_qual(deg, qual)?)
    }
}

impl FromStr for Pitch {
    type Err = err::ParsePitchError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(idx) = s.find('_') {
            let opitch_src = &s[..idx];
            let octave_src = &s[idx + 1..];
            let opitch: OPitch = opitch_src.parse()?;
            let octave = i8::from_str_radix(octave_src, 10)?;
            Ok(Pitch::from_opitch_and_octave(opitch, octave))
        } else {
            Ok(OPitch::from_str(s).map(Pitch::from)?)
        }
    }
}

impl Interval {
    fn from_str_positive(s: &str) -> Result<(Self, bool), err::ParseIntervalError> {
        let (mut qual, deg) = split_qual_and_deg(s)?;
        let sign = if qual.ends_with('-') {
            qual = &qual[..qual.len() - 1];
            false
        } else if qual.ends_with('+') {
            qual = &qual[..qual.len() - 1];
            true
        } else {
            true
        };
        let qual: IntervalQual = qual.parse()?;
        let deg: IntervalDeg = (i8::from_str_radix(deg, 10)? - 1).into();
        use IntervalQual::*;
        use OIntervalDeg::*;
        let odeg = OIntervalDeg::from(deg);
        match (qual, odeg) {
            (Major | Minor, Unison | Fourth | Fifth)
            | (Perfect, Second | Third | Sixth | Seventh) => {
                Err(InvalidOInterval { deg: odeg, qual }.into())
            }
            _ => Ok((Interval { deg, qual }, sign)),
        }
    }
}

impl FromStr for Interval {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('+') {
            let (result, sign) = Interval::from_str_positive(&s[1..])?;
            if sign { Ok(result) } else { Ok(-result) }
        } else if s.starts_with('-') {
            let (result, sign) = Interval::from_str_positive(&s[1..])?;
            if sign { Ok(-result) } else { Ok(result) }
        } else if let Some(idx) = s.find('_') {
            let ointerval_src = &s[..idx];
            let octave_src = &s[idx + 1..];
            let ointerval: OInterval = ointerval_src.parse()?;
            let octave: i8 = i8::from_str_radix(octave_src, 10)?;
            Ok(Interval::from_ointerval_and_octave(ointerval, octave))
        } else {
            let (result, sign) = Interval::from_str_positive(s)?;
            if sign { Ok(result) } else { Ok(-result) }
        }
    }
}
