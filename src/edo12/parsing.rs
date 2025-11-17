use std::str::FromStr;

use anyhow::{anyhow, bail};
use itertools::Itertools as _;
use uncased::AsUncased as _;

use super::base::{Acci, OPitch, STEP_NAMES, Step};

impl FromStr for Step {
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
        if s.is_empty() {
            bail!("Empty input");
        } else {
            let mut chars = s.chars();
            let step = chars
                .peeking_take_while(|&ch| ch.is_ascii_alphabetic())
                .collect::<String>();
            let step = step.parse::<Step>()?;
            let acci = chars.as_str();
            let acci = acci.parse::<Acci>()?;
            Ok(OPitch::new(step, acci))
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