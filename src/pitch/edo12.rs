pub use fantazia_lib::pitch::edo12::*;

#[cfg(feature = "proc-macro")]
pub use fantazia_proc_macro::{opitch, pitch, acci, ostep, step};

#[cfg(test)]
mod test {
    use crate::traits::Co5Order;
    #[test]
    fn test_acci_display() {
        use crate::pitch::edo12::acci;
        let accidentals = [
            acci!(-3), // triple flat
            acci!(-2), // double flat
            acci!(-1), // flat
            acci!(),  // natural
            acci!("+"),  // sharp
            acci!(2),  // double sharp
            acci!(3),  // triple sharp
        ];
        accidentals
            .into_iter()
            .for_each(|acci| println!("{}", acci));
    }
    #[test]
    fn test_step_display() {
        use crate::pitch::edo12::{OStep, ostep};
        use OStep::*;
        let steps = [
            C, D, E, F, G, A, B,
            ostep!(C), ostep!(D), ostep!(E), ostep!(F), ostep!(G), ostep!(A), ostep!(B),
            ostep!("C"), ostep!("D"), ostep!("E"), ostep!("F"), ostep!("G"), ostep!("A"), ostep!("B"),
            ostep!("do"), ostep!("ut"), ostep!("re"), ostep!("mi"), ostep!("fa"), ostep!("sol"), ostep!("la"), ostep!("si"), ostep!("ti"),
        ];
        steps.into_iter().for_each(|step| println!("{}", step));
    }
    #[test]
    fn test_opitch_parse() {
        use crate::pitch::edo12::opitch;
        println!("{}", opitch!("F+"));
    }
    #[test]
    fn test_opitch_arith() {
        use crate::pitch::edo12::opitch;
        println!("{}", opitch!("E") + opitch!("E-"));
        println!("{}", opitch!("E-") + opitch!("E-"));
        println!("{}", opitch!("E") + opitch!("E"));
    }
    #[test]
    fn test_interval() {
        use crate::pitch::edo12::{opitch, OInterval};
        let interval: OInterval = (opitch!("D-") - opitch!("E")).into();
        println!("{}", interval);
    }
    #[test]
    fn test_from_co5_order() {
        use crate::pitch::edo12::OPitch;
        let n_range = -14..=14;
        n_range.for_each(|n| {
            let p = OPitch::from_co5_order(n);
            println!("{} {}", n, p);
            assert_eq!(p.co5_order(), n);
        });
    }
    #[test]
    fn test_interval_parsing() {
        use crate::pitch::edo12::OInterval;
        let valid_intervals = [
            "P1", "m2", "M2", "m3", "M3", "P4", "A4", "d5", "P5", "m6", "M6", "m7", "M7",
        ];
        let invalid_intervals = ["M1", "m1", "P2", "P3", "M4", "m4", "M5", "m5", "P6", "P7"];
        for s in valid_intervals {
            assert!(dbg!(s.parse::<OInterval>()).is_ok())
        }
        for s in invalid_intervals {
            assert!(dbg!(s.parse::<OInterval>()).is_err())
        }
    }

    #[test]
    fn test_compound_interval() {
        use crate::pitch::edo12::{Interval, pitch};
        let interval: Interval = pitch!("E_-1").into();
        println!("{}", interval);
    }

    #[test]
    fn test_pitch_arith() {
        use crate::pitch::edo12::pitch;
        println!("{}", pitch!("E_0") + pitch!("E-_0"));
    }
}