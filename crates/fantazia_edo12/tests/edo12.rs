//! End-to-end tests of the public **12-EDO** API.
//!
//! Unlike the unit tests inside the crate, these tests only use the public interface, so they also
//! serve as executable documentation for downstream users.

use fantazia_core::pitch::{GeneralPitch, SpecificPitch, ToFrequency};
use fantazia_edo12::{
    acci::Acci,
    acci_pref::AcciPref,
    interval::{Interval, IntervalQual, OInterval},
    pitch::{OPitch, Pitch},
    step::{OStep, Step},
    tone::{OTone, Tone},
};

#[test]
fn circle_of_fifths_closes_on_an_augmented_seventh() {
    // Twelve perfect fifths do not return to C, but to B sharp.
    let mut pitch = OPitch::C;
    for _ in 0..12 {
        pitch += OInterval::PERFECT_FIFTH;
    }
    assert_eq!(pitch, OPitch::B.with_acci(1));
    assert!(pitch.is_enharmonic(&OPitch::C));
    assert_eq!(pitch.co5_order(), 12);

    // Five steps down the circle of fifths is a flat key: C -> F -> Bb -> Eb -> Ab -> Db.
    assert_eq!(OPitch::from_co5_order(-5), OPitch::D_FLAT);
    assert_eq!(OPitch::D_FLAT.co5_order(), -5);
}

#[test]
fn intervals_compose_and_invert() {
    // A major third plus a minor third is a perfect fifth.
    assert_eq!(
        OInterval::MAJOR_THIRD + OInterval::MINOR_THIRD,
        OInterval::PERFECT_FIFTH
    );

    // Inverting a major third gives a minor sixth, and adding the two gives an octave.
    let inverted = -OInterval::MAJOR_THIRD;
    assert_eq!(inverted, OInterval::MINOR_SIXTH);
    assert_eq!(
        OInterval::MAJOR_THIRD.put_in_octave(0) + inverted.put_in_octave(0),
        OInterval::PERFECT_UNISON.put_in_octave(1)
    );

    // Quality distinguishes sizes that share a number.
    let major_third = Interval::from(Pitch::E_0);
    let minor_third = Interval::from(Pitch::E_FLAT_0);
    assert_eq!(major_third.qual(), IntervalQual::MAJOR);
    assert_eq!(minor_third.qual(), IntervalQual::MINOR);
    assert_eq!(major_third.step(), Step::E_0);
    assert_eq!(minor_third.step(), Step::E_0);
}

#[test]
fn transposing_a_scale_keeps_diatonic_spelling() {
    let c_major = [
        OPitch::C,
        OPitch::D,
        OPitch::E,
        OPitch::F,
        OPitch::G,
        OPitch::A,
        OPitch::B,
    ];
    let d_major = c_major.map(|pitch| pitch + OInterval::MAJOR_SECOND);
    assert_eq!(
        d_major,
        [
            OPitch::D,
            OPitch::E,
            OPitch::F_SHARP,
            OPitch::G,
            OPitch::A,
            OPitch::B,
            OPitch::C_SHARP,
        ]
    );

    // The same scale, one step down, is C major again.
    let c_major_again = d_major.map(|pitch| pitch - OInterval::MAJOR_SECOND);
    assert_eq!(c_major_again, c_major);
}

#[test]
fn major_scales_are_spelled_by_accidental_preference() {
    // D flat major uses five flats.
    let db_tonic = OPitch::D_FLAT.tone.generalize();
    let db_major = [0, 2, 4, 5, 7, 9, 11].map(|offset| {
        (db_tonic + OTone::new(offset).unwrap())
            .to_opitch_in_key(OPitch::D_FLAT, AcciPref::ALL_FLAT)
    });
    assert_eq!(
        db_major.map(|pitch| pitch.to_string()),
        ["D-", "E-", "F", "G-", "A-", "B-", "C"]
    );

    // B major uses five sharps. The preference decides how black keys are spelled.
    let b_major = [0, 2, 4, 5, 7, 9, 11].map(|offset| {
        (OTone::B + OTone::new(offset).unwrap()).to_opitch_in_key(OPitch::B, AcciPref::ALL_SHARP)
    });
    assert_eq!(
        b_major.map(|pitch| pitch.to_string()),
        ["B", "C+", "D+", "E", "F+", "G+", "A+"]
    );
}

#[test]
fn display_and_parse_round_trip() {
    let pitches = ["C", "C+", "D-", "F++", "B-", "G[+4]", "A[-5]", "C+++"];
    for src in pitches {
        let pitch = src.parse::<OPitch>().unwrap();
        assert_eq!(pitch.to_string(), src, "`{src}`");
    }

    let pitches = ["C_0", "C+_1", "D-_-1", "B_2", "F+_-2"];
    for src in pitches {
        let pitch = src.parse::<Pitch>().unwrap();
        assert_eq!(pitch.to_string(), src, "`{src}`");
    }

    let intervals = ["P1", "m3", "M3", "P5", "A4", "d5", "M10", "-M3", "P8"];
    for src in intervals {
        let interval = src.parse::<Interval>().unwrap();
        assert_eq!(interval.to_string(), src, "`{src}`");
    }
}

#[test]
fn octaves_double_the_frequency() {
    // Frequencies are relative to middle C.
    assert!((Pitch::C_0.to_frequency().get() - 1.0).abs() < 1e-12);
    assert!((Pitch::C_1.to_frequency().get() - 2.0).abs() < 1e-12);
    assert!((Pitch::C_M1.to_frequency().get() - 0.5).abs() < 1e-12);

    // A semitone is the twelfth root of two, and a perfect fifth is seven of them.
    let semitone = Tone::new(1).to_frequency().get();
    assert!((semitone - 2f64.powf(1.0 / 12.0)).abs() < 1e-12);
    assert!((OPitch::G.to_frequency().get() - 2f64.powf(7.0 / 12.0)).abs() < 1e-12);

    // The same pitch one octave up always doubles the frequency.
    for tone in -12i8..=12 {
        let lower = Tone::new(tone).to_frequency().get();
        let upper = Tone::new(tone + 12).to_frequency().get();
        assert!((upper - 2.0 * lower).abs() < 1e-12, "tone {tone}");
    }
}

#[test]
fn steps_transpose_and_respell() {
    // Adding a step transposes by the diatonic interval from C.
    assert_eq!(OPitch::C + Step::new(2), OPitch::E);
    assert_eq!(OPitch::C + Step::new(4), OPitch::G);
    assert_eq!(OPitch::C - OStep::E, OPitch::A_FLAT);

    // Accidentals participate in pitch arithmetic.
    assert_eq!(
        OPitch::C_SHARP + Acci::SHARP,
        OPitch::C.with_acci(Acci::DOUBLE_SHARP)
    );
    assert_eq!(
        OPitch::C.with_acci(Acci::DOUBLE_SHARP) - Acci::SHARP,
        OPitch::C_SHARP
    );

    // Specific pitches keep their octave when transposed.
    assert_eq!(Step::C_1.diatonic_tone(), Tone::C_1);
    assert_eq!(Pitch::new_diatonic(Step::G_M1), Pitch::G_M1);
}

#[test]
fn ergonomic_constants_and_aliases() {
    assert_eq!(OStep::DO, OStep::C);
    assert_eq!(OStep::SOL, OStep::G);
    assert_eq!(OStep::FIFTH, OStep::G);
    assert_eq!(OPitch::C.acci(), Acci::NATURAL);
    assert_eq!(Pitch::C_1.generalize(), OPitch::C);
    assert_eq!(Pitch::C_1.octave(), 1);
    assert_eq!(
        OInterval::PERFECT_FIFTH.pitch().put_in_octave(0),
        Pitch::G_0
    );
}
