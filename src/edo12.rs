mod arith;
mod base;
mod constants;
mod cmp;
mod interval;
mod parsing;
mod repr;

pub use base::{Acci, OPitch, Pitch, OStep, Step};
pub use interval::{SimpleIntervalDeg, IntervalQual, SimpleInterval, Interval, IntervalDeg, AcciByQual};
pub use parsing::opitch;

#[cfg(test)]
mod test {
    use std::str::FromStr as _;

    use crate::traits::Co5Order;

    use super::*;
    #[test]
    fn test_acci_display() {
        let accidentals = [
            Acci(-3), // triple flat
            Acci(-2), // double flat
            Acci(-1), // flat
            Acci(0),  // natural
            Acci(1),  // sharp
            Acci(2),  // double sharp
            Acci(3),  // triple sharp
        ];
        accidentals
            .into_iter()
            .for_each(|acci| println!("{}", acci));
    }
    #[test]
    fn test_step_display() {
        use OStep::*;
        let steps = [C, D, E, F, G, A, B];
        steps.into_iter().for_each(|step| println!("{}", step));
    }
    #[test]
    fn test_opitch_parse() {
        println!("{}", OPitch::from_str("F+").unwrap());
    }
    #[test]
    fn test_opitch_arith() {
        println!("{}", opitch!("E") + opitch!("E-"));
        println!("{}", opitch!("E-") + opitch!("E-"));
        println!("{}", opitch!("E") + opitch!("E"));
    }
    #[test]
    fn test_interval() {
        let interval: SimpleInterval = (opitch!("D-") - opitch!("E")).into();
        println!("{}", interval);
    }
    #[test]
    fn test_from_co5_order() {
        let n_range = -14..=14;
        n_range.for_each(|n| {
            let p = OPitch::from_co5_order(n);
            println!("{} {}", n, p);
            assert_eq!(p.co5_order(), n);
        });
    }
    #[test]
    fn test_interval_parsing() {
        let valid_intervals = ["P1", "m2", "M2", "m3", "M3", "P4", "A4", "d5", "P5", "m6", "M6", "m7", "M7"];
        let invalid_intervals = ["M1", "m1", "P2", "P3", "M4", "m4", "M5", "m5", "P6", "P7"];
        for s in valid_intervals {
            assert!(dbg!(s.parse::<SimpleInterval>()).is_ok())
        }
        for s in invalid_intervals {
            assert!(dbg!(s.parse::<SimpleInterval>()).is_err())
        }
    }

    #[test]
    fn test_compound_interval() {
        let interval: Interval = Pitch::from_str("E_-1").unwrap().into();
        println!("{}", interval);
    }
}
