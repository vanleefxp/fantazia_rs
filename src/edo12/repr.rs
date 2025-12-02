use std::fmt::{Debug, Display, Formatter, Write};

use malachite_base::num::arithmetic::traits::Abs;
use malachite_base::num::basic::traits::Zero;

use super::base::{Acci, OPitch, OStep, Pitch, PitchNotation as _, Step};
use super::interval::{IntervalQual, SimpleInterval, SimpleIntervalDeg};
use crate::edo12::Interval;

impl Display for OStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (ostep, octave) = self.into_ostep_and_octave();
        write!(f, "{}_{}", ostep, octave)
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

impl Display for SimpleIntervalDeg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", (*self) as u8 + 1)
    }
}

impl Display for SimpleInterval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.qual, self.deg)
    }
}

impl Display for Pitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (opitch, octave) = self.into_opitch_and_octave();
        write!(f, "{}_{}", opitch, octave)
    }
}

impl Interval {
    fn fmt_positive(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.qual, self.deg.0 + 1)
    }
}

impl Display for Interval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self >= &Self::ZERO {
            self.fmt_positive(f)
        } else {
            f.write_char('-').and_then(|_| self.abs().fmt_positive(f))
        }
    }
}
