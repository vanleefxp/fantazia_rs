use std::fmt::{Debug, Display, Formatter};

use super::base::{Acci, OPitch, Step};
use super::interval::{IntervalDeg, IntervalQual, SimpleInterval};

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

impl Display for IntervalQual {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use IntervalQual::*;
        match self {
            Perfect => write!(f, "P"),
            Major => write!(f, "M"),
            Minor => write!(f, "m"),
            &Augmented(n @ ..=3) => {
                for _ in 0..n {
                    write!(f, "A")?
                }
                Ok(())
            }
            &Augmented(n) => write!(f, "[A*{n}]"),
            &Diminished(n @ ..=3) => {
                for _ in 0..n {
                    write!(f, "d")?
                }
                Ok(())
            }
            &Diminished(n) => write!(f, "[d*{n}]"),
        }
    }
}

impl Display for IntervalDeg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", (*self) as u8 + 1)
    }
}

impl Display for SimpleInterval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.qual, self.deg)
    }
}
