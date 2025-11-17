use std::fmt::{Debug, Display, Formatter};

use super::base::{Acci, OPitch, Step};

impl Display for Step {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Display for Acci {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            0 => write!(f, "[]"),
            n @ 0..=3 => {
                for _ in 0..n {
                    write!(f, "+")?;
                }
                Ok(())
            }
            n @ -3..0 => {
                for _ in 0..-n {
                    write!(f, "-")?;
                }
                Ok(())
            }
            n if n > 0 => write!(f, "[{n:+}]"),
            n => write!(f, "[{n}]"),
        }
    }
}

impl Display for OPitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_diatonic() {
            write!(f, "{}", self.step)
        } else {
            write!(f, "{}{}", self.step, self.acci())
        }
    }
}
