use std::str::FromStr;

use anyhow::{anyhow, bail};
use itertools::Itertools as _;
use uncased::AsUncased as _;

use super::base::{Acci, OPitch, OStep, Pitch, STEP_NAMES, Step};
use super::interval::{IntervalQual, OInterval, OIntervalDeg, Interval, IntervalDeg};

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

impl FromStr for Step {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(idx) = s.find('_') {
            let ostep = (&s[..idx]).parse::<OStep>()?;
            let octave: i8 = (&s[idx + 1..]).parse()?;
            Ok(Step(ostep as i8 + octave * 12))
        } else {
            Ok(s.parse::<OStep>()?.into())
        }
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

impl FromStr for OIntervalDeg {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deg_plus_1: u8 = s.parse()?;
        let deg = OIntervalDeg::try_from(
            deg_plus_1
                .checked_sub(1)
                .ok_or_else(|| anyhow!("0 is not a valid interval degree."))?,
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
                                    let n: u8 = u8::from_str_radix(chars.dropping_back(1).as_str(), 10)?;
                                    Ok(Augmented(n))
                                } else {
                                    bail!("Invalid interval quality: {s}");
                                }
                            }
                            'd' => {
                                if let Some('*') = chars.next() {
                                    let n: u8 = u8::from_str_radix(chars.dropping_back(1).as_str(), 10)?;
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

fn split_qual_and_deg(s: &str) -> Result<(&str, &str), anyhow::Error> {
    let (qual, deg) = s
            .char_indices()
            .rev()
            .take_while(|&(_, ch)| ch.is_ascii_digit())
            .last()
            .map(|(idx, _)| (&s[..idx], &s[idx..]))
            .ok_or_else(|| anyhow!("Invalid interval format: {s}"))?;
    Ok((qual, deg))
}

impl FromStr for OInterval {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (qual, deg) = split_qual_and_deg(s)?;
        let qual: IntervalQual = qual.parse()?;
        let deg: OIntervalDeg = deg.parse()?;
        OInterval::try_from_deg_and_qual(deg, qual)
    }
}

impl FromStr for Pitch {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(idx) = s.find('_') {
            let opitch_src = &s[..idx];
            let octave_src = &s[idx + 1..];
            let opitch: OPitch = opitch_src.parse()?;
            let octave = i8::from_str_radix(octave_src, 10)?;
            Ok(Pitch::from_opitch_and_octave(opitch, octave))
        } else {
            OPitch::from_str(s).map(Pitch::from)
        }
    }
}

impl Interval {
    fn from_str_positive(s: &str) -> Result<(Self, bool), anyhow::Error> {
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
        match (qual, OIntervalDeg::from(deg)) {
            (Major | Minor, Unison | Fourth | Fifth)
            | (Perfect, Second | Third | Sixth | Seventh) => {
                bail!("{s} is not a valid interval.")
            }
            _ => Ok((Interval { deg, qual }, sign))
        }
    }
}

impl FromStr for Interval {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('+') {
            let (result, sign) = Interval::from_str_positive(&s[1..])?;
            if sign {
                Ok(result)
            } else {Ok(-result)}
        } else if s.starts_with('-') {
            let (result, sign) = Interval::from_str_positive(&s[1..])?;
            if sign {
                Ok(-result)}
            else {Ok(result)}
        } else if let Some(idx) = s.find('_') {
            let ointerval_src = &s[..idx];
            let octave_src = &s[idx + 1..];
            let ointerval: OInterval = ointerval_src.parse()?;
            let octave: i8 = i8::from_str_radix(octave_src, 10)?;
            Ok(Interval::from_ointerval_and_octave(ointerval, octave))
        } else {
            let (result, sign) = Interval::from_str_positive(s)?;
            if sign {
                Ok(result)
            } else {
                Ok(-result)
            }
        }
    }
}