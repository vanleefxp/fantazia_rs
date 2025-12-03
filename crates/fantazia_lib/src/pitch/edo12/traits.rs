use malachite_base::num::basic::traits::Zero;

pub trait Co5Order {
    type Output;
    fn co5_order(self) -> Self::Output;
}

pub trait FromCo5Order<T> {
    fn from_co5_order(t: T) -> Self;
}

pub trait PitchNotation {
    type Step;
    type OStep;
    type Acci: Zero + PartialEq;
    type OTone;
    type Tone: PartialEq;
    type Octave;

    fn step(&self) -> Self::Step;
    fn tone(&self) -> Self::Tone;
    fn ostep(&self) -> Self::OStep;
    fn octave(&self) -> Self::Octave;
    fn octave_by_tone(&self) -> Self::Octave;
    fn otone(&self) -> Self::OTone;
    fn acci(&self) -> Self::Acci;

    fn is_diatonic(&self) -> bool {
        self.acci() == Self::Acci::ZERO
    }

    fn is_enharmonic(&self, other: &Self) -> bool {
        self.tone() == other.tone()
    }
}