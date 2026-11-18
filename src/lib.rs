//! # `fantazia`: semantic music computation package

pub use fantazia_core::*;

#[cfg(feature = "edo12")]
pub mod edo12 {
    pub use fantazia_edo12::*;
}
