use std::str::FromStr;

use anyhow::anyhow;
use malachite_base::num::arithmetic::traits::CheckedLogBase2;
use phf::phf_map;

use crate::rhythm::BinaryDuration;

static COMMON_DURATIONS: phf::Map<&'static str, i8> = phf_map! {
    "1/8" | "/8" => -3,
    "1/4" | "/4" => -2,
    "1/2" | "/2" => -1,
    "1" => 0,
    "2" => 1,
    "4" => 2,
    "8" => 3,
    "16" => 4,
    "32" => 5,
    "64" => 6,
    "128" => 7,
    "256" => 8,
    "512" => 9,
    "1024" => 10,
};

impl FromStr for BinaryDuration {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (duration_src, dots) = if let Some((i1, i2)) = s
            .char_indices()
            .rev()
            .enumerate()
            .take_while(|&(_, (_, ch))| ch == '.')
            .last()
            .map(|(i1, (i2, _))| (i1, i2))
        {
            (&s[..i2], u8::try_from(i1 + 1)?)
        } else {
            (&s[..], 0u8)
        };
        match COMMON_DURATIONS.get(duration_src) {
            Some(&kind) => Ok(BinaryDuration { kind, dots }),
            None => {
                let (sign, duration_src) = if duration_src.starts_with("1/") {
                    (false, &duration_src[2..])
                } else if duration_src.starts_with('/') {
                    (false, &duration_src[1..])
                } else {
                    (true, duration_src)
                };
                let duration: u128 = duration_src.parse()?;
                let kind: i8 = duration
                    .checked_log_base_2()
                    .ok_or_else(|| anyhow!("Invalid duration"))?
                    .try_into()?;
                let kind = if sign { kind } else { -kind };
                Ok(BinaryDuration { kind, dots })
            }
        }
    }
}
