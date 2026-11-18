#![feature(
    const_trait_impl,
    const_convert,
    const_ops,
    const_cmp,
    const_default,
    const_clone,
    derive_const,
    pattern_types,
    pattern_type_macro,
    pattern_type_range_trait
)]
#![allow(internal_features)]

use cyclique::new_modulus_type;

new_modulus_type! {
    #[derive(cyclique::Debug)]
    pub struct SmallNums(10);

    #[derive(cyclique::Debug)]
    pub struct LargeNums(240);

    #[derive(cyclique::Debug)]
    pub struct Exact(256);

    #[derive(cyclique::Debug)]
    pub struct Greater(400);

    #[derive(cyclique::Debug)]
    pub struct Huge(65537);

    #[derive(cyclique::Debug)]
    pub struct Trivial(1);

    #[derive(cyclique::Debug)]
    pub enum DayOfWeek { Sunday, #[default] Monday, Tuesday, Wednesday, Thursday, Friday, Saturday }

    #[derive(cyclique::Debug)]
    pub enum Month { Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct, Nov, Dec }

    #[derive(cyclique::Debug)]
    pub enum Unique { One }
}

#[test]
fn test_size_of() {
    assert_eq!(size_of::<SmallNums>(), 1);
    assert_eq!(size_of::<LargeNums>(), 1);
    assert_eq!(size_of::<Option<SmallNums>>(), 1);
    assert_eq!(size_of::<Option<LargeNums>>(), 1);
    assert_eq!(size_of::<Exact>(), 1);
    assert_eq!(size_of::<Option<Exact>>(), 2);
    assert_eq!(size_of::<Greater>(), 2);
    assert_eq!(size_of::<Trivial>(), 0);
    // assert_eq!(size_of::<Unique>(), 0);
    assert_eq!(size_of::<Huge>(), 4);
}

#[test]
fn test_add() {
    assert_eq!(SmallNums::_1 + SmallNums::_2, SmallNums::_3);
    assert_eq!(SmallNums::_3 + SmallNums::_8, SmallNums::_1);
    assert_eq!(LargeNums::_140 + LargeNums::_150, LargeNums::_50);
    assert_eq!(Exact::new(255) + Exact::new(2), Exact::new(1));
}

#[test]
fn test_debug() {
    assert_eq!(format!("{:?}", SmallNums::_1), "SmallNums(1)");
    assert_eq!(format!("{:?}", LargeNums::_100), "LargeNums(100)");
    assert_eq!(format!("{:?}", Exact::new(100)), "Exact(100)");
}
