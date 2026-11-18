use proc_macro2::Literal;
use quote::{ToTokens, quote};

#[derive(Debug, Copy, Hash)]
#[derive_const(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ReprType {
    U8,
    U16,
    U32,
    U64,
    U128,
}

#[derive(Debug, Copy, Hash)]
#[derive_const(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ReprTypeOrZst {
    Zst,
    NonZero(ReprType),
}

const impl ReprType {
    pub fn from_variant_count(count: u128) -> Option<Self> {
        if count == 1 {
            None
        } else if count == 0 {
            Some(Self::U128)
        } else if count <= (1 << 8) {
            Some(Self::U8)
        } else if count <= (1 << 16) {
            Some(Self::U16)
        } else if count <= (1 << 32) {
            Some(Self::U32)
        } else if count <= (1 << 64) {
            Some(Self::U64)
        } else {
            Some(Self::U128)
        }
    }

    pub fn next(&self) -> Option<Self> {
        use ReprType::*;
        match *self {
            U8 => Some(U16),
            U16 => Some(U32),
            U32 => Some(U64),
            U64 => Some(U128),
            U128 => None,
        }
    }

    /// Number of bits of the representation type.
    pub fn bits(&self) -> u32 {
        use ReprType::*;
        match *self {
            U8 => 8,
            U16 => 16,
            U32 => 32,
            U64 => 64,
            U128 => 128,
        }
    }

    /// Largest value representable by the representation type.
    pub fn max_value(&self) -> u128 {
        let bits = self.bits();
        if bits == 128 {
            u128::MAX
        } else {
            (1u128 << bits) - 1
        }
    }
}

const impl ReprTypeOrZst {
    fn from_variant_count(count: u128) -> Self {
        if let Some(count) = ReprType::from_variant_count(count) {
            Self::NonZero(count)
        } else {
            Self::Zst
        }
    }
}

impl ToTokens for ReprType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use ReprType::*;
        tokens.extend(match *self {
            U8 => quote!(u8),
            U16 => quote!(u16),
            U32 => quote!(u32),
            U64 => quote!(u64),
            U128 => quote!(u128),
        })
    }
}

const impl From<ReprType> for ReprTypeOrZst {
    fn from(value: ReprType) -> Self {
        ReprTypeOrZst::NonZero(value)
    }
}

const impl TryFrom<ReprTypeOrZst> for ReprType {
    type Error = ();
    fn try_from(value: ReprTypeOrZst) -> Result<Self, Self::Error> {
        if let ReprTypeOrZst::NonZero(value) = value {
            Ok(value)
        } else {
            Err(())
        }
    }
}

impl ToTokens for ReprTypeOrZst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Self::NonZero(value) = self {
            value.to_tokens(tokens);
        } else {
            tokens.extend(quote!(()));
        }
    }
}

#[derive(Debug, Copy, Hash)]
#[derive_const(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Divisor {
    count: u128,
    repr_type: ReprTypeOrZst,
}

const impl Divisor {
    pub fn new(value: u128) -> Self {
        Self {
            count: value,
            repr_type: ReprTypeOrZst::from_variant_count(value),
        }
    }
    #[inline(always)]
    pub fn count(&self) -> u128 {
        self.count
    }
    #[inline(always)]
    pub fn repr_type(&self) -> ReprTypeOrZst {
        self.repr_type
    }
    pub fn is_boundary(&self) -> bool {
        let value = self.count;
        value <= 1
            || value == (1 << 8)
            || value == (1 << 16)
            || value == (1 << 32)
            || value == (1 << 64)
    }

    pub fn is_power_of_two(&self) -> bool {
        self.count.is_power_of_two()
    }
}

#[derive(Debug, Copy, Hash)]
#[derive_const(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct NonBoundaryDivisor {
    count: u128,
    repr_type: ReprType,
}

const impl NonBoundaryDivisor {
    #[inline(always)]
    pub fn count(&self) -> u128 {
        self.count
    }
    #[inline(always)]
    pub fn repr_type(&self) -> ReprType {
        self.repr_type
    }
}

const impl TryFrom<Divisor> for NonBoundaryDivisor {
    type Error = ();
    fn try_from(value: Divisor) -> Result<Self, Self::Error> {
        if value.is_boundary() {
            return Err(());
        }
        let Divisor { count, repr_type } = value;
        Ok(Self {
            count,
            repr_type: repr_type.try_into()?,
        })
    }
}

impl ToTokens for NonBoundaryDivisor {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(repr_literal(self.repr_type, self.count).into_token_stream());
    }
}

/// Build a suffixed integer literal with the given representation type.
pub fn repr_literal(repr: ReprType, value: u128) -> Literal {
    use ReprType::*;
    match repr {
        U8 => Literal::u8_suffixed(value as u8),
        U16 => Literal::u16_suffixed(value as u16),
        U32 => Literal::u32_suffixed(value as u32),
        U64 => Literal::u64_suffixed(value as u64),
        U128 => Literal::u128_suffixed(value),
    }
}

/// Precomputed constants for Montgomery multiplication modulo a 128-bit odd
/// modulus, or modulo the odd part of a 128-bit even modulus.
#[derive(Debug, Clone, Copy)]
pub enum Montgomery {
    /// The modulus itself is odd.
    Odd { nprime: u128, r2: u128 },
    /// The modulus is `2^shift * odd` with `odd > 1` odd.
    Even {
        odd: u128,
        mask: u128,
        nprime: u128,
        r2: u128,
        odd_inverse: u128,
    },
}

/// Compute the constants needed by [`Montgomery`] for a modulus that does not
/// fit in a `u64` and is not a power of two.
pub fn montgomery_constants(modulus: u128) -> Montgomery {
    if modulus & 1 == 1 {
        let nprime = inverse_mod_2_128(modulus).wrapping_neg();
        Montgomery::Odd {
            nprime,
            r2: pow2_mod(256, modulus),
        }
    } else {
        let shift = modulus.trailing_zeros();
        let odd = modulus >> shift;
        let mask = (1u128 << shift) - 1;
        let inverse = inverse_mod_2_128(odd);
        Montgomery::Even {
            odd,
            mask,
            nprime: inverse.wrapping_neg(),
            r2: pow2_mod(256, odd),
            odd_inverse: inverse & mask,
        }
    }
}

/// Multiplicative inverse of an odd number modulo `2^128`.
fn inverse_mod_2_128(value: u128) -> u128 {
    // Newton iteration doubles the number of correct bits: starting from the
    // inverse modulo 2, seven steps reach the inverse modulo 2^128.
    let mut inverse = 1u128;
    for _ in 0..7 {
        inverse = inverse.wrapping_mul(2u128.wrapping_sub(value.wrapping_mul(inverse)));
    }
    inverse
}

/// Compute `2^exponent mod modulus`.
fn pow2_mod(exponent: u32, modulus: u128) -> u128 {
    let mut result = 1u128 % modulus;
    for _ in 0..exponent {
        let distance = modulus - result;
        result = if result >= distance {
            result - distance
        } else {
            result + result
        };
    }
    result
}
