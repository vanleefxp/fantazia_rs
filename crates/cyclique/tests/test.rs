#![feature(
    const_trait_impl,
    const_convert,
    const_ops,
    const_cmp,
    const_default,
    const_clone,
    derive_const,
    pattern_types,
    pattern_type_range_trait
)]
#![allow(internal_features)]

use std::num::NonZero;

use cyclique::{ModulusType, new_modulus_type};

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
    pub struct Pow2(128);

    #[derive(cyclique::Debug)]
    pub struct Pow2Wide(1099511627776);

    #[derive(cyclique::Debug)]
    pub struct Boundary16(65536);

    #[derive(cyclique::Debug)]
    pub struct Boundary32(4294967296);

    #[derive(cyclique::Debug)]
    pub struct Boundary64(18446744073709551616);

    #[derive(cyclique::Debug)]
    pub struct Boundary128(0);

    #[derive(cyclique::Debug)]
    pub struct BigOdd(18446744073709551617);

    #[derive(cyclique::Debug)]
    pub struct BigEven(36893488147419103234);

    #[derive(cyclique::Debug)]
    pub struct BigEvenLargeShift(255211775190703847597530955573826158592);

    #[derive(cyclique::Debug)]
    pub struct BigPow2(1267650600228229401496703205376);

    #[derive(cyclique::Debug)]
    pub struct MaxOdd(340282366920938463463374607431768211455);

    #[derive(cyclique::Debug)]
    pub enum DayOfWeek { Sunday, #[default] Monday, Tuesday, Wednesday, Thursday, Friday, Saturday }

    #[derive(cyclique::Debug)]
    pub enum Month { Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct, Nov, Dec }

    #[derive(cyclique::Debug)]
    pub enum Unique { One }
}

fn add_ref(a: u128, b: u128, modulus: u128) -> u128 {
    if modulus == 0 {
        return a.wrapping_add(b);
    }
    let (sum, carry) = a.overflowing_add(b);
    if carry || sum >= modulus {
        sum.wrapping_sub(modulus)
    } else {
        sum
    }
}

fn sub_ref(a: u128, b: u128, modulus: u128) -> u128 {
    if modulus == 0 {
        return a.wrapping_sub(b);
    }
    let (difference, borrow) = a.overflowing_sub(b);
    if borrow {
        difference.wrapping_add(modulus)
    } else {
        difference
    }
}

fn neg_ref(a: u128, modulus: u128) -> u128 {
    if modulus == 0 {
        a.wrapping_neg()
    } else if a == 0 {
        0
    } else {
        modulus - a
    }
}

fn mul_ref_slow(mut a: u128, mut b: u128, modulus: u128) -> u128 {
    if modulus == 0 {
        return a.wrapping_mul(b);
    }
    a %= modulus;
    let mut result = 0;
    while b > 0 {
        if b & 1 == 1 {
            result = add_ref(result, a, modulus);
        }
        a = add_ref(a, a, modulus);
        b >>= 1;
    }
    result
}

fn residue_unsigned(value: u128, modulus: u128) -> u128 {
    if modulus == 0 { value } else { value % modulus }
}

fn residue_signed(value: i128, modulus: u128) -> u128 {
    if modulus == 0 {
        return value as u128;
    }
    if value >= 0 {
        (value as u128) % modulus
    } else {
        let magnitude = value.unsigned_abs() % modulus;
        if magnitude == 0 {
            0
        } else {
            modulus - magnitude
        }
    }
}

macro_rules! check_self_ops {
    ($ty:ty, $n:expr) => {{
        let modulus: u128 = $n;
        let last = if modulus == 0 { u128::MAX } else { modulus - 1 };
        let values = [0u128, 1, 2, modulus / 2, last];
        for &a in &values {
            for &b in &values {
                let x = <$ty>::from(a);
                let y = <$ty>::from(b);
                let ax = (*x) as u128;
                let bx = (*y) as u128;
                assert_eq!((*(x + y)) as u128, add_ref(ax, bx, modulus));
                assert_eq!((*(x - y)) as u128, sub_ref(ax, bx, modulus));
                assert_eq!((*(x * y)) as u128, mul_ref_slow(ax, bx, modulus));
                assert_eq!((*(-x)) as u128, neg_ref(ax, modulus));

                let mut z = x;
                z += y;
                assert_eq!((*z) as u128, add_ref(ax, bx, modulus));
                let mut z = x;
                z -= y;
                assert_eq!((*z) as u128, sub_ref(ax, bx, modulus));
                let mut z = x;
                z *= y;
                assert_eq!((*z) as u128, mul_ref_slow(ax, bx, modulus));
            }
        }
    }};
}

macro_rules! check_unsigned_primitive {
    ($ty:ty, $n:expr, $p:ty, $vals:expr) => {{
        let modulus: u128 = $n;
        let last = if modulus == 0 { u128::MAX } else { modulus - 1 };
        for &a in &[0u128, 1, last] {
            for &rhs in $vals.iter() {
                let x = <$ty>::from(a);
                let ax = (*x) as u128;
                let bx = residue_unsigned(rhs as u128, modulus);
                assert_eq!((*(x + rhs)) as u128, add_ref(ax, bx, modulus));
                assert_eq!((*(x - rhs)) as u128, sub_ref(ax, bx, modulus));
                assert_eq!((*(x * rhs)) as u128, mul_ref_slow(ax, bx, modulus));

                let mut z = x;
                z += rhs;
                assert_eq!((*z) as u128, add_ref(ax, bx, modulus));
                let mut z = x;
                z -= rhs;
                assert_eq!((*z) as u128, sub_ref(ax, bx, modulus));
                let mut z = x;
                z *= rhs;
                assert_eq!((*z) as u128, mul_ref_slow(ax, bx, modulus));
            }
        }
    }};
}

macro_rules! check_signed_primitive {
    ($ty:ty, $n:expr, $p:ty, $vals:expr) => {{
        let modulus: u128 = $n;
        let last = if modulus == 0 { u128::MAX } else { modulus - 1 };
        for &a in &[0u128, 1, last] {
            for &rhs in $vals.iter() {
                let x = <$ty>::from(a);
                let ax = (*x) as u128;
                let bx = residue_signed(rhs as i128, modulus);
                assert_eq!((*(x + rhs)) as u128, add_ref(ax, bx, modulus));
                assert_eq!((*(x - rhs)) as u128, sub_ref(ax, bx, modulus));
                assert_eq!((*(x * rhs)) as u128, mul_ref_slow(ax, bx, modulus));

                let mut z = x;
                z += rhs;
                assert_eq!((*z) as u128, add_ref(ax, bx, modulus));
                let mut z = x;
                z -= rhs;
                assert_eq!((*z) as u128, sub_ref(ax, bx, modulus));
                let mut z = x;
                z *= rhs;
                assert_eq!((*z) as u128, mul_ref_slow(ax, bx, modulus));
            }
        }
    }};
}

macro_rules! check_all_primitives {
    ($ty:ty, $n:expr) => {{
        check_unsigned_primitive!($ty, $n, u8, [0u8, 1, 2, 127, 128, 255]);
        check_unsigned_primitive!($ty, $n, u16, [0u16, 1, 255, 256, u16::MAX]);
        check_unsigned_primitive!($ty, $n, u32, [0u32, 1, 65535, 65536, u32::MAX]);
        check_unsigned_primitive!($ty, $n, u64, [0u64, 1, u32::MAX as u64, u64::MAX]);
        check_unsigned_primitive!($ty, $n, u128, [0u128, 1, u64::MAX as u128, u128::MAX]);
        check_unsigned_primitive!($ty, $n, usize, [0usize, 1, usize::MAX]);
        check_signed_primitive!($ty, $n, i8, [i8::MIN, -2, -1, 0, 1, 2, i8::MAX]);
        check_signed_primitive!($ty, $n, i16, [i16::MIN, -1, 0, 1, i16::MAX]);
        check_signed_primitive!($ty, $n, i32, [i32::MIN, -1, 0, 1, i32::MAX]);
        check_signed_primitive!($ty, $n, i64, [i64::MIN, -1, 0, 1, i64::MAX]);
        check_signed_primitive!($ty, $n, i128, [i128::MIN, -1, 0, 1, i128::MAX]);
        check_signed_primitive!($ty, $n, isize, [isize::MIN, -1, 0, 1, isize::MAX]);
    }};
}

macro_rules! check_type {
    ($ty:ty, $n:expr) => {{
        check_self_ops!($ty, $n);
        check_all_primitives!($ty, $n);
    }};
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn next_u128(&mut self) -> u128 {
        ((self.next() as u128) << 64) | self.next() as u128
    }

    fn below(&mut self, modulus: u128) -> u128 {
        if modulus == 0 {
            self.next_u128()
        } else {
            self.next_u128() % modulus
        }
    }
}

macro_rules! check_random {
    ($ty:ty, $n:expr, $iterations:expr) => {{
        let modulus: u128 = $n;
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        for _ in 0..$iterations {
            let a = rng.below(modulus);
            let b = rng.below(modulus);
            let x = <$ty>::from(a);
            let y = <$ty>::from(b);
            let ax = (*x) as u128;
            let bx = (*y) as u128;
            assert_eq!((*(x + y)) as u128, add_ref(ax, bx, modulus));
            assert_eq!((*(x - y)) as u128, sub_ref(ax, bx, modulus));
            assert_eq!((*(x * y)) as u128, mul_ref_slow(ax, bx, modulus));
            assert_eq!((*(-x)) as u128, neg_ref(ax, modulus));
        }
    }};
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
    assert_eq!(size_of::<Unique>(), 0);
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
fn test_sub_mul_neg() {
    assert_eq!(SmallNums::_1 - SmallNums::_2, SmallNums::_9);
    assert_eq!(SmallNums::_2 - SmallNums::_9, SmallNums::_3);
    assert_eq!(SmallNums::_3 * SmallNums::_7, SmallNums::_1);
    assert_eq!(-SmallNums::_0, SmallNums::_0);
    assert_eq!(-SmallNums::_3, SmallNums::_7);
    assert_eq!(Exact::new(200) * Exact::new(2), Exact::new(144));
}

#[test]
fn test_primitive_ops() {
    assert_eq!(SmallNums::_1 + 2u8, SmallNums::_3);
    assert_eq!(SmallNums::_1 + 12u8, SmallNums::_3);
    assert_eq!(SmallNums::_1 - 2i8, SmallNums::_9);
    assert_eq!(SmallNums::_3 * -1i32, SmallNums::_7);
    assert_eq!(SmallNums::_3 * 10u128, SmallNums::_0);
    assert_eq!(SmallNums::_3 * -10i128, SmallNums::_0);
    assert_eq!(SmallNums::_3 + 5usize, SmallNums::_8);
    assert_eq!(SmallNums::_3 - 5isize, SmallNums::_8);
    assert_eq!(-SmallNums::_3 + 2i8, SmallNums::_9);
}

#[test]
fn test_nonzero_ops() {
    let value = NonZero::new(3u8).unwrap();
    assert_eq!(SmallNums::_2 * value, SmallNums::_6);
    assert_eq!(SmallNums::from(value), SmallNums::_3);
    assert_eq!(SmallNums::from(13u8), SmallNums::_3);
    assert_eq!(Exact::from(255u8), Exact::new(255));
}

#[test]
fn test_debug() {
    assert_eq!(format!("{:?}", SmallNums::_1), "SmallNums(1)");
    assert_eq!(format!("{:?}", LargeNums::_100), "LargeNums(100)");
    assert_eq!(format!("{:?}", Exact::new(100)), "Exact(100)");
    assert_eq!(format!("{:?}", DayOfWeek::Monday), "DayOfWeek::Monday");
}

#[test]
fn test_zst_ops() {
    let z = Unique::default();
    assert_eq!(z + z, z);
    assert_eq!(z - z, z);
    assert_eq!(z * z, z);
    assert_eq!(-z, z);
    assert_eq!(z + 123u8, z);
    assert_eq!(z - 123i8, z);
    assert_eq!(z * -123i128, z);
    assert_eq!(z * NonZero::new(7u16).unwrap(), z);

    let mut assigned = z;
    assigned += z;
    assigned -= z;
    assigned *= z;
    assigned += 1usize;
    assert_eq!(assigned, z);

    assert_eq!(Trivial::from(99u8), Trivial);
}

#[test]
fn test_modulus_type_trait() {
    assert_eq!(<SmallNums as ModulusType>::DIVISOR, 10u8);
    assert_eq!(<Exact as ModulusType>::DIVISOR, 0u8);
    assert_eq!(<BigOdd as ModulusType>::DIVISOR, 18446744073709551617u128);
    assert_eq!(<Boundary128 as ModulusType>::DIVISOR, 0u128);
}

#[test]
fn test_non_boundary_types() {
    check_type!(SmallNums, 10);
    check_type!(LargeNums, 240);
    check_type!(Greater, 400);
    check_type!(Huge, 65537);
    check_type!(Pow2, 128);
    check_type!(Pow2Wide, 1099511627776);
    check_type!(DayOfWeek, 7);
    check_type!(Month, 12);
}

#[test]
fn test_boundary_types() {
    check_type!(Exact, 256);
    check_type!(Boundary16, 65536);
    check_type!(Boundary32, 4294967296);
    check_type!(Boundary64, 18446744073709551616);
    check_type!(Boundary128, 0);
}

#[test]
fn test_wide_moduli() {
    check_type!(BigOdd, 18446744073709551617);
    check_type!(BigEven, 36893488147419103234);
    check_type!(BigEvenLargeShift, 255211775190703847597530955573826158592);
    check_type!(BigPow2, 1267650600228229401496703205376);
    check_type!(MaxOdd, 340282366920938463463374607431768211455);
}

#[test]
fn test_random_ops() {
    check_random!(SmallNums, 10, 200);
    check_random!(LargeNums, 240, 200);
    check_random!(Greater, 400, 200);
    check_random!(Huge, 65537, 200);
    check_random!(Pow2, 128, 200);
    check_random!(Pow2Wide, 1099511627776, 200);
    check_random!(Exact, 256, 200);
    check_random!(Boundary16, 65536, 200);
    check_random!(Boundary32, 4294967296, 200);
    check_random!(Boundary64, 18446744073709551616, 200);
    check_random!(Boundary128, 0, 200);
    check_random!(BigOdd, 18446744073709551617, 500);
    check_random!(BigEven, 36893488147419103234, 500);
    check_random!(
        BigEvenLargeShift,
        255211775190703847597530955573826158592,
        500
    );
    check_random!(BigPow2, 1267650600228229401496703205376, 200);
    check_random!(MaxOdd, 340282366920938463463374607431768211455, 500);
}

#[test]
fn test_wide_mul_identities() {
    // 2^64 + 1, so 2^64 is a square root of unity.
    let two_64 = BigOdd::from(18446744073709551616u128);
    assert_eq!(*(two_64 * two_64), 1);
    assert_eq!(*(two_64 * two_64 * two_64), 18446744073709551616);

    // Modulo 2^128 - 1, the value 2^128 - 2 is -1, whose square is 1.
    let minus_one = MaxOdd::from(u128::MAX - 1);
    assert_eq!(*(minus_one * minus_one), 1);

    let big_pow2 = BigPow2::from(1267650600228229401496703205375u128);
    assert_eq!(*(big_pow2 * big_pow2), 1);
}

const CONST_SUM: SmallNums = SmallNums::_1 + SmallNums::_2;
const CONST_PRODUCT: SmallNums = SmallNums::_7 * 3u8;
const CONST_DIFFERENCE: SmallNums = SmallNums::_2 - 5i8;
const CONST_SIGNED_PRODUCT: SmallNums = SmallNums::_3 * -3i8;
const CONST_WIDE_ADD: SmallNums = SmallNums::_3 + 1000u128;
const CONST_BOUNDARY_PRODUCT: Exact = Exact::new(200) * 2u64;
const CONST_BIG_SQUARE: BigOdd =
    BigOdd::from(18_446_744_073_709_551_616u128) * BigOdd::from(18_446_744_073_709_551_616u128);
const CONST_BIG_EVEN_PRODUCT: BigEven = BigEven::from(3u128) * BigEven::from(5u128);
const CONST_BIG_POW2_PRODUCT: BigPow2 = BigPow2::from(2u128) * BigPow2::from(3u128);

#[test]
fn test_const_ops() {
    assert_eq!(CONST_SUM, SmallNums::_3);
    assert_eq!(CONST_PRODUCT, SmallNums::_1);
    assert_eq!(CONST_DIFFERENCE, SmallNums::_7);
    assert_eq!(CONST_SIGNED_PRODUCT, SmallNums::_1);
    assert_eq!(CONST_WIDE_ADD, SmallNums::_3);
    assert_eq!(CONST_BOUNDARY_PRODUCT, Exact::new(144));
    assert_eq!(*CONST_BIG_SQUARE, 1);
    assert_eq!(*CONST_BIG_EVEN_PRODUCT, 15);
    assert_eq!(*CONST_BIG_POW2_PRODUCT, 6);
}

#[test]
fn test_from_reduces() {
    assert_eq!(*SmallNums::from(123_456_789u128), 9);
    assert_eq!(*SmallNums::from(-123_456_789i128), 1);
    assert_eq!(*SmallNums::from(-1i8), 9);
    assert_eq!(*Exact::from(511u16), 255);
    assert_eq!(*Exact::from(-1i8), 255);
    assert_eq!(*Boundary128::from(-1i8), u128::MAX);
    assert_eq!(*Pow2::from(255u8), 127);
    assert_eq!(*BigOdd::from(20_000_000_000_000_000_000u128), {
        20_000_000_000_000_000_000u128 % 18_446_744_073_709_551_617u128
    });
}

#[cfg(feature = "serde")]
mod serde_support {
    use super::*;

    #[test]
    fn serde_round_trip_for_pattern_type() {
        assert_eq!(serde_json::to_string(&SmallNums::_9).unwrap(), "9");
        assert_eq!(
            serde_json::from_str::<SmallNums>("9").unwrap(),
            SmallNums::_9
        );
        assert!(serde_json::from_str::<SmallNums>("10").is_err());
    }

    #[test]
    fn serde_round_trip_for_boundary_type() {
        let value = Exact::new(255);
        assert_eq!(serde_json::to_string(&value).unwrap(), "255");
        assert_eq!(serde_json::from_str::<Exact>("255").unwrap(), value);
    }

    #[test]
    fn serde_round_trip_for_enum() {
        assert_eq!(serde_json::to_string(&DayOfWeek::Friday).unwrap(), "5");
        assert_eq!(
            serde_json::from_str::<DayOfWeek>("5").unwrap(),
            DayOfWeek::Friday
        );
        assert!(serde_json::from_str::<DayOfWeek>("7").is_err());
    }

    #[test]
    fn serde_round_trip_for_zero_sized_type() {
        assert_eq!(serde_json::to_string(&Trivial).unwrap(), "0");
        assert_eq!(serde_json::from_str::<Trivial>("0").unwrap(), Trivial);
        assert_eq!(serde_json::from_str::<Trivial>("1").unwrap(), Trivial);
    }

    #[test]
    fn serde_round_trip_for_wide_types() {
        let value = Greater::from(399u16);
        assert_eq!(serde_json::to_string(&value).unwrap(), "399");
        assert_eq!(serde_json::from_str::<Greater>("399").unwrap(), value);
        assert!(serde_json::from_str::<Greater>("400").is_err());

        let value = BigOdd::from(123u128);
        assert_eq!(serde_json::to_string(&value).unwrap(), "123");
        assert_eq!(serde_json::from_str::<BigOdd>("123").unwrap(), value);

        let value = Boundary64::new(u64::MAX);
        assert_eq!(
            serde_json::from_str::<Boundary64>(&value.to_string()).unwrap(),
            value
        );
    }
}

#[cfg(feature = "rkyv")]
mod rkyv_support {
    use super::*;
    use rkyv::{from_bytes, rancor::Error, to_bytes};

    #[test]
    fn rkyv_round_trip_for_pattern_type() {
        let bytes = to_bytes::<Error>(&SmallNums::_9).unwrap();
        let value: SmallNums = from_bytes::<SmallNums, Error>(&bytes).unwrap();
        assert_eq!(value, SmallNums::_9);
    }

    #[test]
    fn rkyv_rejects_out_of_range_residue() {
        let bytes = to_bytes::<Error>(&10u8).unwrap();
        assert!(from_bytes::<SmallNums, Error>(&bytes).is_err());

        let bytes = to_bytes::<Error>(&7u8).unwrap();
        assert!(from_bytes::<DayOfWeek, Error>(&bytes).is_err());
    }

    #[test]
    fn rkyv_round_trip_for_boundary_type() {
        let value = Exact::new(255);
        let bytes = to_bytes::<Error>(&value).unwrap();
        let value: Exact = from_bytes::<Exact, Error>(&bytes).unwrap();
        assert_eq!(value, Exact::new(255));
    }

    #[test]
    fn rkyv_round_trip_for_enum() {
        let bytes = to_bytes::<Error>(&DayOfWeek::Friday).unwrap();
        let value: DayOfWeek = from_bytes::<DayOfWeek, Error>(&bytes).unwrap();
        assert_eq!(value, DayOfWeek::Friday);
    }

    #[test]
    fn rkyv_round_trip_for_zero_sized_type() {
        let value = Trivial;
        let bytes = to_bytes::<Error>(&value).unwrap();
        let value: Trivial = from_bytes::<Trivial, Error>(&bytes).unwrap();
        assert_eq!(value, Trivial);
    }

    #[test]
    fn rkyv_round_trip_for_wide_types() {
        let value = Greater::from(399u16);
        let bytes = to_bytes::<Error>(&value).unwrap();
        let value: Greater = from_bytes::<Greater, Error>(&bytes).unwrap();
        assert_eq!(value, Greater::from(399u16));

        let value = BigOdd::from(123u128);
        let bytes = to_bytes::<Error>(&value).unwrap();
        let value: BigOdd = from_bytes::<BigOdd, Error>(&bytes).unwrap();
        assert_eq!(value, BigOdd::from(123u128));

        let value = Boundary64::new(u64::MAX);
        let bytes = to_bytes::<Error>(&value).unwrap();
        let value: Boundary64 = from_bytes::<Boundary64, Error>(&bytes).unwrap();
        assert_eq!(value, Boundary64::new(u64::MAX));
    }

    #[test]
    fn rkyv_rejects_out_of_range_wide_residue() {
        let bytes = to_bytes::<Error>(&400u16).unwrap();
        assert!(from_bytes::<Greater, Error>(&bytes).is_err());
    }
}
