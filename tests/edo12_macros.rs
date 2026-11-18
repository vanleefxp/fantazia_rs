#![cfg(feature = "macro")]

//! Tests for the literal construction macros re-exported by `fantazia::edo12`.

use fantazia::edo12::{
    acci, acci::Acci, interval, interval::Interval, ointerval, opitch, ostep, otone, pitch,
    pitch::OPitch, pitch::Pitch, step, step::OStep, step::Step, tone, tone::OTone, tone::Tone,
};

#[test]
fn ostep_literals() {
    assert_eq!(ostep!(C), OStep::C);
    assert_eq!(ostep!("C"), OStep::C);
    assert_eq!(ostep!(do), OStep::C);
    assert_eq!(ostep!(sol), OStep::G);
    assert_eq!(ostep!(7), OStep::C);
    assert_eq!(ostep!(-1), OStep::B);
    assert_eq!(ostep!(7u8), OStep::C);
}

#[test]
fn step_literals() {
    assert_eq!(step!(C_0), Step::C_0);
    assert_eq!(step!("C_1"), Step::C_1);
    assert_eq!(step!(B_ - 1), Step::B_M1);
    assert_eq!(step!(7), Step::C_1);
    assert_eq!(step!(-7), Step::C_M1);
    assert_eq!(step!(g_1), Step::G_1);
}

#[test]
fn opitch_literals() {
    assert_eq!(opitch!(F+), OPitch::F_SHARP);
    assert_eq!(opitch!("F+"), OPitch::F_SHARP);
    assert_eq!(opitch!(B-), OPitch::B_FLAT);
    assert_eq!(opitch!(C), OPitch::C);
    assert_eq!(opitch!(C[+4]).to_string(), "C[+4]");
    assert_eq!(opitch!(D--).to_string(), "D--");
}

#[test]
fn pitch_literals() {
    assert_eq!(pitch!(C_1), Pitch::C_1);
    assert_eq!(pitch!("F+_0"), Pitch::F_SHARP_0);
    assert_eq!(pitch!(B_ - 1), Pitch::B_M1);
    assert_eq!(pitch!(G[+3]_2), pitch!(G+++_2));
    assert_eq!(pitch!(G[+3]_2).acci(), Acci::TRIPLE_SHARP);
}

#[test]
fn tone_literals() {
    assert_eq!(tone!(C_0), Tone::C_0);
    assert_eq!(tone!("C_1"), Tone::C_1);
    assert_eq!(tone!(C+), Tone::new(1));
    assert_eq!(tone!(6), Tone::new(6));

    assert_eq!(otone!(C), OTone::C);
    assert_eq!(otone!(F+), OTone::F_SHARP);
    assert_eq!(otone!("F+"), OTone::F_SHARP);
    assert_eq!(otone!(C_1), OTone::C);
    assert_eq!(otone!(13), OTone::C_SHARP);
}

#[test]
fn acci_literals() {
    assert_eq!(acci!(++), Acci::DOUBLE_SHARP);
    assert_eq!(acci!("--"), Acci::DOUBLE_FLAT);
    assert_eq!(acci!([+4]), Acci::new(4));
    assert_eq!(acci!(-3), Acci::TRIPLE_FLAT);
    assert_eq!(acci!(=), Acci::NATURAL);
}

#[test]
fn interval_literals() {
    assert_eq!(
        ointerval!(M3),
        fantazia::edo12::interval::OInterval::MAJOR_THIRD
    );
    assert_eq!(
        ointerval!("P5"),
        fantazia::edo12::interval::OInterval::PERFECT_FIFTH
    );
    assert_eq!(
        ointerval!(d5),
        fantazia::edo12::interval::OInterval::DIMINISHED_FIFTH
    );

    assert_eq!(interval!(M10), interval!(M3_1));
    assert_eq!(interval!("M10"), interval!(M10));
    assert_eq!(interval!(M3).to_string(), "M3");
    assert_eq!(interval!(-M3).to_string(), "-M3");
    assert_eq!(interval!(M128).to_string(), "M128");
}

#[test]
fn literals_are_expressions() {
    // The macros can be nested in ordinary expressions and participate in type inference.
    let chord = [opitch!(C), opitch!(E), opitch!(G)];
    assert_eq!(chord[1] - chord[0], opitch!(E));

    assert_eq!(step!(C_0) + step!(7), step!(C_1));

    let third: Interval = interval!(M3);
    assert_eq!(third.step(), Step::E_0);
}

#[test]
fn literals_in_const_items() {
    // These expand to plain constant constructors.
    const STEP: Step = step!(C_1);
    const OSTEP: OStep = ostep!(G);
    const ACCI: Acci = acci!(++);
    const INTERVAL: fantazia::edo12::interval::OInterval = ointerval!(M3);

    assert_eq!(STEP, Step::C_1);
    assert_eq!(OSTEP, OStep::G);
    assert_eq!(ACCI, Acci::DOUBLE_SHARP);
    assert_eq!(INTERVAL, fantazia::edo12::interval::OInterval::MAJOR_THIRD);
}
