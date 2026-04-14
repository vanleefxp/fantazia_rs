use std::fmt::{Display, Formatter, Write};

use crate::rhythm::BinaryDuration;

const COMMON_DURATIONS: [&str; 11] = [
    "1", "2", "4", "8", "16", "32", "64", "128", "256", "512", "1024",
];

impl Display for BinaryDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            0..=10 => {
                f.write_str(COMMON_DURATIONS[self.kind as usize])?;
            }
            -10..0 => {
                f.write_char('/')?;
                f.write_str(COMMON_DURATIONS[(-self.kind - 1) as usize])?;
            }
            n if n > 0 => {
                write!(f, "{}", 1u128 << n)?;
            }
            n => {
                write!(f, "/{}", 1u128 << (-n))?;
            }
        }
        for _ in 0..self.dots {
            f.write_char('.')?;
        }
        Ok(())
    }
}
