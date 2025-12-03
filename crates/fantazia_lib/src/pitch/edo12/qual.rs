use std::cmp::Ordering;

use malachite_base::num::arithmetic::traits::Sign as _;

use super::{
    Acci, Interval, IntervalDeg, IntervalQual, OInterval, OIntervalDeg, OPitch, OStep, Pitch, Step,
    traits::{AbsQual, AcciByQual, Qual},
};

impl Qual for OInterval {
    fn qual(&self) -> IntervalQual {
        self.qual
    }
}

impl Qual for Interval {
    fn qual(&self) -> IntervalQual {
        self.qual
    }
}

impl Qual for OPitch {
    fn qual(&self) -> IntervalQual {
        use IntervalQual::*;
        use OStep::*;
        let tone_diff = self.tone - self.step.diatonic_tone();
        match self.step {
            C | F | G => match tone_diff {
                0 => Perfect,
                n if n > 0 => Augmented(n as u8),
                n => Diminished((-n) as u8),
            },
            _ => match tone_diff {
                0 => Major,
                -1 => Minor,
                n if n > 0 => Augmented(n as u8),
                n => Diminished((-n - 1) as u8),
            },
        }
    }
}

impl Qual for Pitch {
    fn qual(&self) -> IntervalQual {
        OPitch::from(*self).qual()
    }
}

impl AbsQual for Pitch {
    fn abs_qual(&self) -> IntervalQual {
        use IntervalQual::*;
        use Ordering::*;
        match self.sign() {
            Less => -self.qual(),
            Equal => Perfect,
            Greater => self.qual(),
        }
    }
}

impl AbsQual for Interval {
    fn abs_qual(&self) -> IntervalQual {
        use Ordering::*;
        match self.sign() {
            Greater | Equal => self.qual(),
            Less => -self.qual(),
        }
    }
}

impl AcciByQual for OStep {
    fn acci_by_qual(&self, qual: IntervalQual) -> Option<Acci> {
        use IntervalQual::*;
        use OStep::*;
        match self {
            C | F | G => match qual {
                Perfect => Some(Acci::NATURAL),
                Augmented(n) => Some(Acci(n as i8)),
                Diminished(n) => Some(Acci(-(n as i8))),
                _ => None,
            },
            _ => match qual {
                Major => Some(Acci::NATURAL),
                Minor => Some(Acci::FLAT),
                Augmented(n) => Some(Acci(n as i8)),
                Diminished(n) => Some(Acci(-(n as i8) - 1)),
                _ => None,
            },
        }
    }
}

impl AcciByQual for Step {
    fn acci_by_qual(&self, qual: IntervalQual) -> Option<Acci> {
        OStep::from(*self).acci_by_qual(qual)
    }
}

impl AcciByQual for OIntervalDeg {
    fn acci_by_qual(&self, qual: IntervalQual) -> Option<Acci> {
        OStep::from(*self).acci_by_qual(qual)
    }
}

impl AcciByQual for IntervalDeg {
    fn acci_by_qual(&self, qual: IntervalQual) -> Option<Acci> {
        Step::from(*self).acci_by_qual(qual)
    }
}
