mod arith;
mod base;
mod constants;
mod parsing;
mod repr;

pub use base::{Acci, OPitch, Step};
pub use parsing::opitch;

#[cfg(test)]
mod test {
    use std::str::FromStr as _;

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
        use Step::*;
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
}
