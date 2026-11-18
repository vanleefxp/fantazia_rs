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
pub enum SignedReprType {
    I8,
    I16,
    I32,
    I64,
    I128,
}

#[derive(Debug, Copy, Hash)]
#[derive_const(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ReprTypeOrZST {
    ZST,
    NonZero(ReprType),
}

const impl ReprType {
    pub const ALL: &[Self] = &[Self::U8, Self::U16, Self::U32, Self::U64, Self::U128];
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

    pub fn to_signed(&self) -> SignedReprType {
        use ReprType::*;
        use SignedReprType::*;
        match *self {
            U8 => I8,
            U16 => I16,
            U32 => I32,
            U64 => I64,
            U128 => I128,
        }
    }
}

const impl SignedReprType {
    pub fn to_unsigned(&self) -> ReprType {
        use ReprType::*;
        use SignedReprType::*;
        match *self {
            I8 => U8,
            I16 => U16,
            I32 => U32,
            I64 => U64,
            I128 => U128,
        }
    }
}

const impl ReprTypeOrZST {
    fn from_variant_count(count: u128) -> Self {
        if let Some(count) = ReprType::from_variant_count(count) {
            Self::NonZero(count)
        } else {
            Self::ZST
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

impl ToTokens for SignedReprType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use SignedReprType::*;
        tokens.extend(match *self {
            I8 => quote!(i8),
            I16 => quote!(i16),
            I32 => quote!(i32),
            I64 => quote!(i64),
            I128 => quote!(i128),
        })
    }
}

const impl From<ReprType> for ReprTypeOrZST {
    fn from(value: ReprType) -> Self {
        ReprTypeOrZST::NonZero(value)
    }
}

const impl TryFrom<ReprTypeOrZST> for ReprType {
    type Error = ();
    fn try_from(value: ReprTypeOrZST) -> Result<Self, Self::Error> {
        if let ReprTypeOrZST::NonZero(value) = value {
            Ok(value)
        } else {
            Err(())
        }
    }
}

impl ToTokens for ReprTypeOrZST {
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
    repr_type: ReprTypeOrZST,
}

const impl Divisor {
    pub fn new(value: u128) -> Self {
        Self {
            count: value,
            repr_type: ReprTypeOrZST::from_variant_count(value),
        }
    }
    #[allow(dead_code)]
    #[inline(always)]
    pub fn count(&self) -> u128 {
        self.count
    }
    #[inline(always)]
    pub fn repr_type(&self) -> ReprTypeOrZST {
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
}

#[derive(Debug, Copy, Hash)]
#[derive_const(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct NonBoundaryDivisor {
    count: u128,
    repr_type: ReprType,
}

const impl NonBoundaryDivisor {
    #[allow(dead_code)]
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
        let count = self.count;
        use ReprType::*;
        match self.repr_type {
            U8 => tokens.extend(Literal::u8_suffixed(count as u8).into_token_stream()),
            U16 => tokens.extend(Literal::u16_suffixed(count as u16).into_token_stream()),
            U32 => tokens.extend(Literal::u32_suffixed(count as u32).into_token_stream()),
            U64 => tokens.extend(Literal::u64_suffixed(count as u64).into_token_stream()),
            U128 => tokens.extend(Literal::u128_suffixed(count).into_token_stream()),
        }
    }
}
