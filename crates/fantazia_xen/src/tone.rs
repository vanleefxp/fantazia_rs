// use core::pat::RangePattern;
// use std::{
//     fmt::{self, Debug, Display},
//     mem,
// };

// #[derive(Copy)]
// #[derive_const(Clone)]
// pub struct OToneEDO<const N: u8 = 11>(pattern_type!(u8 is 0..=N));

// fn otone_edo<const EDO: u8>(value: pattern_type!(u8 is 0..EDO)) -> OToneEDO<{ EDO - 1 }> {
//     OToneEDO::<{ EDO - 1 }>(value)
// }

// impl<const EDO: u8> OToneEDO<EDO> {
//     pub const unsafe fn new_unchecked(value: u8) -> Self {
//         unsafe { Self(mem::transmute(value)) }
//     }
//     pub const fn as_u8(&self) -> u8 {
//         unsafe { mem::transmute(self.0) }
//     }
// }

// // const impl Default for OToneEDO

// impl<const EDO: u8> Debug for OToneEDO<EDO> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_tuple(&format!("OToneEDO<{EDO}>"))
//             .field(&self.as_u8())
//             .finish()
//     }
// }

// impl<const EDO: u8> Display for OToneEDO<EDO> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.as_u8())
//     }
// }

// pub struct ToneEDO<const EDO: u8>(i16);
