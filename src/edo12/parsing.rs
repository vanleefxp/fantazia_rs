use std::str::FromStr;

use anyhow::{anyhow, bail};
use itertools::Itertools as _;
use uncased::AsUncased as _;

use super::base::{Acci, OPitch, OStep, Pitch, STEP_NAMES};
use super::interval::{IntervalQual, SimpleInterval, SimpleIntervalDeg};

impl FromStr for OStep {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = s.as_uncased();
        STEP_NAMES
            .get(&key)
            .cloned()
            .ok_or_else(|| anyhow!("Invalid step name: {}", s))
    }
}

impl FromStr for Acci {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self::NATURAL)
        } else if s.starts_with('[') {
            if s.ends_with(']') {
                let s = &s[1..s.len() - 1];
                if s.is_empty() {
                    Ok(Acci::NATURAL)
                } else {
                    Ok(Acci(s.parse::<i8>()?))
                }
            } else {
                bail!("Unclosed bracket in accidental string")
            }
        } else {
            let mut acci: i8 = 0;
            for ch in s.chars() {
                match ch {
                    '+' => acci += 1,
                    '-' => acci -= 1,
                    _ => bail!("Invalid character in accidental string"),
                }
            }
            Ok(Acci(acci))
        }
    }
}

impl FromStr for OPitch {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.find(|ch: char| !ch.is_ascii_alphabetic()) {
            Some(idx) => Ok(OPitch::new((&s[..idx]).parse()?, (&s[idx..]).parse()?)),
            None => Ok(s.parse::<OStep>()?.into()),
        }
    }
}

impl FromStr for SimpleIntervalDeg {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deg_plus_1: u8 = s.parse()?;
        let deg = SimpleIntervalDeg::try_from(
            deg_plus_1
                .checked_sub(1)
                .ok_or_else(|| anyhow!("0 is not a valid interval degree."))?,
        )?;
        Ok(deg)
    }
}

impl FromStr for IntervalQual {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use IntervalQual::*;
        match s {
            "P" => Ok(Perfect),
            "M" => Ok(Major),
            "m" => Ok(Minor),
            s => {
                let mut chars = s.chars();
                match chars.next().ok_or_else(|| anyhow!("Empty input"))? {
                    '[' => {
                        if !s.ends_with("]") {
                            bail!("Unclosed bracket in interval quality string");
                        }
                        match chars.next().unwrap() {
                            'A' => {
                                if let Some('*') = chars.next() {
                                    let n: u8 = chars.dropping_back(1).as_str().parse()?;
                                    Ok(Augmented(n))
                                } else {
                                    bail!("Invalid interval quality: {s}");
                                }
                            }
                            'd' => {
                                if let Some('*') = chars.next() {
                                    let n: u8 = chars.dropping_back(1).as_str().parse()?;
                                    Ok(Diminished(n))
                                } else {
                                    bail!("Invalid interval quality: {s}");
                                }
                            }
                            _ => bail!("Invalid interval quality: {}", s),
                        }
                    }
                    'A' => {
                        let mut n = 1;
                        while let Some('A') = chars.next() {
                            n += 1;
                        }
                        match chars.next() {
                            None => Ok(Augmented(n)),
                            _ => bail!("Invalid interval quality: {s}"),
                        }
                    }
                    'd' => {
                        let mut n = 1;
                        while let Some('d') = chars.next() {
                            n += 1;
                        }
                        match chars.next() {
                            None => Ok(Diminished(n)),
                            _ => bail!("Invalid interval quality: {s}"),
                        }
                    }
                    _ => bail!("Invalid interval quality: {}", s),
                }
            }
        }
    }
}

impl FromStr for SimpleInterval {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (qual, deg) = s
            .char_indices()
            .rev()
            .take_while(|&(_, ch)| ch.is_ascii_digit())
            .last()
            .map(|(idx, _)| (&s[..idx], &s[idx..]))
            .ok_or_else(|| anyhow!("Invalid interval format: {s}"))?;
        let qual: IntervalQual = qual.parse()?;
        let deg: SimpleIntervalDeg = deg.parse()?;
        use IntervalQual::*;
        use SimpleIntervalDeg::*;
        match (qual, deg) {
            (Major | Minor, Unison | Fourth | Fifth)
            | (Perfect, Second | Third | Sixth | Seventh) => {
                bail!("{qual}{deg} is not a valid simple interval.")
            }
            _ => {}
        }
        Ok(SimpleInterval { deg, qual })
    }
}

impl FromStr for Pitch {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(idx) = s.find('_') {
            let opitch_src = &s[..idx];
            let octave_src = &s[idx + 1..];
            let opitch: OPitch = opitch_src.parse()?;
            let octave: i8 = octave_src.parse()?;
            Ok(Pitch::from_opitch_and_octave(opitch, octave))
        } else {
            OPitch::from_str(s).map(Pitch::from)
        }
    }
}

#[macro_export]
macro_rules! opitch {
    ($src:expr) => {
        OPitch::from_str($src).unwrap()
    };
}

pub use opitch;
