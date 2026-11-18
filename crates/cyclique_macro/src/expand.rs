use proc_macro2::{self as pm2, Literal};
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    Divisor, ReprType, ReprTypeOrZst,
    repr::{Montgomery, montgomery_constants, repr_literal},
    utils::{mem_path, num_path, ops_path},
};

/// A built-in integer type that can take part in an operation with a modulus
/// type.
#[derive(Clone, Copy)]
struct Primitive {
    name: &'static str,
    kind: PrimitiveKind,
}

#[derive(Clone, Copy)]
enum PrimitiveKind {
    /// `Some(bits)` for a fixed-width type, `None` for `usize`.
    Unsigned(Option<u32>),
    /// `Some(bits)` for a fixed-width type, `None` for `isize`.
    Signed(Option<u32>),
}

const PRIMITIVES: [Primitive; 12] = [
    Primitive {
        name: "u8",
        kind: PrimitiveKind::Unsigned(Some(8)),
    },
    Primitive {
        name: "u16",
        kind: PrimitiveKind::Unsigned(Some(16)),
    },
    Primitive {
        name: "u32",
        kind: PrimitiveKind::Unsigned(Some(32)),
    },
    Primitive {
        name: "u64",
        kind: PrimitiveKind::Unsigned(Some(64)),
    },
    Primitive {
        name: "u128",
        kind: PrimitiveKind::Unsigned(Some(128)),
    },
    Primitive {
        name: "usize",
        kind: PrimitiveKind::Unsigned(None),
    },
    Primitive {
        name: "i8",
        kind: PrimitiveKind::Signed(Some(8)),
    },
    Primitive {
        name: "i16",
        kind: PrimitiveKind::Signed(Some(16)),
    },
    Primitive {
        name: "i32",
        kind: PrimitiveKind::Signed(Some(32)),
    },
    Primitive {
        name: "i64",
        kind: PrimitiveKind::Signed(Some(64)),
    },
    Primitive {
        name: "i128",
        kind: PrimitiveKind::Signed(Some(128)),
    },
    Primitive {
        name: "isize",
        kind: PrimitiveKind::Signed(None),
    },
];

impl Primitive {
    fn ty(&self) -> Ident {
        format_ident!("{}", self.name)
    }

    /// The largest non-negative value the primitive can hold, when known at
    /// macro-expansion time.
    fn max_value(&self) -> Option<u128> {
        match self.kind {
            PrimitiveKind::Unsigned(Some(bits)) => Some(max_unsigned(bits)),
            PrimitiveKind::Signed(Some(bits)) => Some(max_signed(bits)),
            PrimitiveKind::Unsigned(None) | PrimitiveKind::Signed(None) => None,
        }
    }

    /// The largest residue a primitive value can map to modulo `modulus`.
    fn residue_bound(&self, modulus: u128) -> u128 {
        let modulus_bound = modulus - 1;
        match self.kind {
            PrimitiveKind::Unsigned(Some(_)) => self
                .max_value()
                .map_or(modulus_bound, |max| max.min(modulus_bound)),
            PrimitiveKind::Unsigned(None) | PrimitiveKind::Signed(_) => modulus_bound,
        }
    }

    fn bits(&self) -> Option<u32> {
        match self.kind {
            PrimitiveKind::Unsigned(bits) | PrimitiveKind::Signed(bits) => bits,
        }
    }
}

fn reduction_name(primitive: &Primitive) -> Ident {
    format_ident!("__cyclique_reduce_{}", primitive.name)
}

/// Whether combining the reduction with the operation itself is worthwhile.
///
/// Widening the operands to the next integer type makes the operation a single
/// modulo. This pays off for 8- and 16-bit representations, where the wider
/// modulo is cheap; larger widths are better served by reducing the primitive
/// first.
fn combined_reduction_worthwhile(repr: ReprType, primitive: &Primitive) -> bool {
    repr.bits() <= 16 && primitive.bits().is_some_and(|bits| bits <= repr.bits())
}

fn next_signed_type(repr: ReprType) -> Option<&'static str> {
    match repr {
        ReprType::U8 => Some("i16"),
        ReprType::U16 => Some("i32"),
        ReprType::U32 | ReprType::U64 | ReprType::U128 => None,
    }
}

fn max_unsigned(bits: u32) -> u128 {
    if bits == 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    }
}

fn max_signed(bits: u32) -> u128 {
    (1u128 << (bits - 1)) - 1
}

/// The three families of modulus types.
#[derive(Clone, Copy)]
enum Class {
    /// Exactly one value, represented without storage.
    Zst,
    /// The modulus covers the whole representation type, so every value of the
    /// representation is valid and wrapping arithmetic is modular arithmetic.
    Boundary(ReprType),
    /// The modulus is strictly smaller than the representation's capacity.
    NonBoundary {
        repr: ReprType,
        modulus: u128,
        power_of_two: bool,
    },
}

impl Class {
    fn of(divisor: Divisor) -> Self {
        match divisor.repr_type() {
            ReprTypeOrZst::Zst => Self::Zst,
            ReprTypeOrZst::NonZero(repr) => {
                if divisor.is_boundary() {
                    Self::Boundary(repr)
                } else {
                    let modulus = divisor.count();
                    Self::NonBoundary {
                        repr,
                        modulus,
                        power_of_two: divisor.is_power_of_two(),
                    }
                }
            }
        }
    }
}

pub fn expand_new_unchecked_impl(
    tokens: &mut pm2::TokenStream,
    type_name: &Ident,
    count: crate::repr::NonBoundaryDivisor,
) {
    let mem = mem_path();
    let repr_type = count.repr_type();
    let count = count.count();
    let valid_range_src = format!("`0..{count}`");
    tokens.extend(quote! {
        const impl #type_name {
            /// Creates a new value directly from its raw representation without checking whether the
            /// value falls in the
            #[doc = #valid_range_src]
            /// range.
            ///
            /// # Safety
            /// Caller must ensure that value falls in the
            #[doc = #valid_range_src]
            /// range. Otherwise it is undefined behavior.
            #[inline(always)]
            unsafe fn new_unchecked(value: #repr_type) -> Self {
                unsafe { #mem::transmute(value) }
            }
        }
    });
}

pub fn expand_ops_impls(tokens: &mut pm2::TokenStream, type_name: &Ident, divisor: Divisor) {
    let class = Class::of(divisor);
    if let ReprTypeOrZst::NonZero(repr) = divisor.repr_type() {
        expand_deref(tokens, type_name, repr);
        expand_modulus_type(tokens, type_name, repr, class);
    }

    match class {
        Class::Zst => expand_zst_ops(tokens, type_name),
        Class::Boundary(repr) => expand_boundary_ops(tokens, type_name, repr),
        Class::NonBoundary {
            repr,
            modulus,
            power_of_two,
        } => expand_non_boundary_ops(tokens, type_name, repr, modulus, power_of_two),
    }
}

fn expand_deref(tokens: &mut pm2::TokenStream, type_name: &Ident, repr: ReprType) {
    let mem = mem_path();
    tokens.extend(quote! {
        const impl ::cyclique::__private::core::ops::Deref for #type_name {
            type Target = #repr;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                // SAFETY: the modulus type has the same memory layout as its
                // representation type
                unsafe { #mem::transmute(self) }
            }
        }
    });
}

fn expand_modulus_type(
    tokens: &mut pm2::TokenStream,
    type_name: &Ident,
    repr: ReprType,
    class: Class,
) {
    let divisor = match class {
        // For boundary moduli the representation wraps around, and zero
        // encodes the fact that the divisor is `2^bits`.
        Class::Boundary(_) => repr_literal(repr, 0),
        Class::NonBoundary { modulus, .. } => repr_literal(repr, modulus),
        Class::Zst => return,
    };
    tokens.extend(quote! {
        impl ::cyclique::ModulusType for #type_name {
            type Repr = #repr;
            const DIVISOR: #repr = #divisor;
        }
    });
}

fn expand_zst_ops(tokens: &mut pm2::TokenStream, type_name: &Ident) {
    let ops = ops_path();
    let num = num_path();

    tokens.extend(quote! {
        const impl #ops::Add for #type_name {
            type Output = Self;
            #[inline(always)]
            fn add(self, _rhs: Self) -> Self { self }
        }

        const impl #ops::AddAssign for #type_name {
            #[inline(always)]
            fn add_assign(&mut self, _rhs: Self) {}
        }

        const impl #ops::Sub for #type_name {
            type Output = Self;
            #[inline(always)]
            fn sub(self, _rhs: Self) -> Self { self }
        }

        const impl #ops::SubAssign for #type_name {
            #[inline(always)]
            fn sub_assign(&mut self, _rhs: Self) {}
        }

        const impl #ops::Mul for #type_name {
            type Output = Self;
            #[inline(always)]
            fn mul(self, _rhs: Self) -> Self { self }
        }

        const impl #ops::MulAssign for #type_name {
            #[inline(always)]
            fn mul_assign(&mut self, _rhs: Self) {}
        }

        const impl #ops::Neg for #type_name {
            type Output = Self;
            #[inline(always)]
            fn neg(self) -> Self { self }
        }
    });

    for primitive in PRIMITIVES {
        let ty = primitive.ty();
        tokens.extend(quote! {
            const impl #ops::Add<#ty> for #type_name {
                type Output = Self;
                #[inline(always)]
                fn add(self, _rhs: #ty) -> Self { self }
            }

            const impl #ops::AddAssign<#ty> for #type_name {
                #[inline(always)]
                fn add_assign(&mut self, _rhs: #ty) {}
            }

            const impl #ops::Sub<#ty> for #type_name {
                type Output = Self;
                #[inline(always)]
                fn sub(self, _rhs: #ty) -> Self { self }
            }

            const impl #ops::SubAssign<#ty> for #type_name {
                #[inline(always)]
                fn sub_assign(&mut self, _rhs: #ty) {}
            }

            const impl #ops::Mul<#ty> for #type_name {
                type Output = Self;
                #[inline(always)]
                fn mul(self, _rhs: #ty) -> Self { self }
            }

            const impl #ops::MulAssign<#ty> for #type_name {
                #[inline(always)]
                fn mul_assign(&mut self, _rhs: #ty) {}
            }

            const impl ::cyclique::__private::core::convert::From<#ty> for #type_name {
                #[inline(always)]
                fn from(_value: #ty) -> Self { Self::default() }
            }

            const impl ::cyclique::__private::core::convert::From<#num::NonZero<#ty>> for #type_name {
                #[inline(always)]
                fn from(_value: #num::NonZero<#ty>) -> Self { Self::default() }
            }

            const impl #ops::Mul<#num::NonZero<#ty>> for #type_name {
                type Output = Self;
                #[inline(always)]
                fn mul(self, _rhs: #num::NonZero<#ty>) -> Self { self }
            }
        });
    }
}

fn expand_boundary_ops(tokens: &mut pm2::TokenStream, type_name: &Ident, repr: ReprType) {
    let ops = ops_path();
    let num = num_path();

    tokens.extend(quote! {
        const impl #ops::Add for #type_name {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self {
                Self::new((*self).wrapping_add(*rhs))
            }
        }

        const impl #ops::AddAssign for #type_name {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        const impl #ops::Sub for #type_name {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self {
                Self::new((*self).wrapping_sub(*rhs))
            }
        }

        const impl #ops::SubAssign for #type_name {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        const impl #ops::Mul for #type_name {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self {
                Self::new((*self).wrapping_mul(*rhs))
            }
        }

        const impl #ops::MulAssign for #type_name {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs;
            }
        }

        const impl #ops::Neg for #type_name {
            type Output = Self;
            #[inline(always)]
            fn neg(self) -> Self {
                Self::new((*self).wrapping_neg())
            }
        }
    });

    for primitive in PRIMITIVES {
        let ty = primitive.ty();
        tokens.extend(quote! {
            const impl #ops::Add<#ty> for #type_name {
                type Output = Self;
                #[inline(always)]
                fn add(self, rhs: #ty) -> Self {
                    Self::new((*self).wrapping_add(rhs as #repr))
                }
            }

            const impl #ops::AddAssign<#ty> for #type_name {
                #[inline(always)]
                fn add_assign(&mut self, rhs: #ty) {
                    *self = *self + rhs;
                }
            }

            const impl #ops::Sub<#ty> for #type_name {
                type Output = Self;
                #[inline(always)]
                fn sub(self, rhs: #ty) -> Self {
                    Self::new((*self).wrapping_sub(rhs as #repr))
                }
            }

            const impl #ops::SubAssign<#ty> for #type_name {
                #[inline(always)]
                fn sub_assign(&mut self, rhs: #ty) {
                    *self = *self - rhs;
                }
            }

            const impl #ops::Mul<#ty> for #type_name {
                type Output = Self;
                #[inline(always)]
                fn mul(self, rhs: #ty) -> Self {
                    Self::new((*self).wrapping_mul(rhs as #repr))
                }
            }

            const impl #ops::MulAssign<#ty> for #type_name {
                #[inline(always)]
                fn mul_assign(&mut self, rhs: #ty) {
                    *self = *self * rhs;
                }
            }

            const impl ::cyclique::__private::core::convert::From<#ty> for #type_name {
                #[inline(always)]
                fn from(value: #ty) -> Self {
                    Self::new(value as #repr)
                }
            }

            const impl ::cyclique::__private::core::convert::From<#num::NonZero<#ty>> for #type_name {
                #[inline(always)]
                fn from(value: #num::NonZero<#ty>) -> Self {
                    Self::new(value.get() as #repr)
                }
            }

            const impl #ops::Mul<#num::NonZero<#ty>> for #type_name {
                type Output = Self;
                #[inline(always)]
                fn mul(self, rhs: #num::NonZero<#ty>) -> Self {
                    Self::new((*self).wrapping_mul(rhs.get() as #repr))
                }
            }
        });
    }
}

fn expand_non_boundary_ops(
    tokens: &mut pm2::TokenStream,
    type_name: &Ident,
    repr: ReprType,
    modulus: u128,
    power_of_two: bool,
) {
    let ops = ops_path();
    let num = num_path();

    let add_mod = add_mod_body(repr, modulus, power_of_two);
    let sub_mod = sub_mod_body(repr, modulus, power_of_two);
    let neg_mod = neg_mod_body(repr, modulus, power_of_two);
    let mul_mod = mul_mod_body(repr, modulus);
    let small_mul = (!power_of_two
        && PRIMITIVES.iter().any(|primitive| {
            !combined_reduction_worthwhile(repr, primitive)
                && product_fits(repr, modulus, primitive.residue_bound(modulus))
        }))
    .then(|| {
        let n = repr_literal(repr, modulus);
        quote! {
            /// Multiplies two residues whose product is known to fit in the
            /// representation type.
            #[inline(always)]
            fn __cyclique_mul_mod_small(a: #repr, b: #repr) -> #repr {
                a.wrapping_mul(b) % #n
            }
        }
    });
    let reductions = PRIMITIVES.iter().map(|primitive| {
        let name = reduction_name(primitive);
        let ty = primitive.ty();
        let reduction = reduce_expr(repr, modulus, power_of_two, *primitive, quote!(value));
        quote! {
            #[inline(always)]
            fn #name(value: #ty) -> #repr {
                #reduction
            }
        }
    });
    let n = repr_literal(repr, modulus);
    let primitive_ops = PRIMITIVES.iter().map(|primitive| {
        let add_name = format_ident!("__cyclique_add_primitive_{}", primitive.name);
        let mul_name = format_ident!("__cyclique_mul_primitive_{}", primitive.name);
        let ty = primitive.ty();
        let reduction = reduction_name(primitive);
        let wide = repr.next();
        let wide_signed = next_signed_type(repr).map(|wide| format_ident!("{}", wide));

        let add_body = if power_of_two {
            quote! { Self::__cyclique_add_mod(a, b as #repr) }
        } else if combined_reduction_worthwhile(repr, primitive) {
            match primitive.kind {
                PrimitiveKind::Unsigned(_) => {
                    if let Some(wide) = wide {
                        quote! { (((a as #wide) + (b as #wide)) % (#n as #wide)) as #repr }
                    } else {
                        quote! { Self::__cyclique_add_mod(a, Self::#reduction(b)) }
                    }
                }
                PrimitiveKind::Signed(_) => {
                    if let Some(wide) = wide_signed.clone() {
                        quote! {
                            (((a as #wide) + (b as #wide)).rem_euclid(#n as #wide)) as #repr
                        }
                    } else {
                        quote! { Self::__cyclique_add_mod(a, Self::#reduction(b)) }
                    }
                }
            }
        } else {
            quote! { Self::__cyclique_add_mod(a, Self::#reduction(b)) }
        };

        let mul_body = if power_of_two {
            quote! { Self::__cyclique_mul_mod(a, b as #repr) }
        } else if combined_reduction_worthwhile(repr, primitive) {
            match primitive.kind {
                PrimitiveKind::Unsigned(_) => {
                    if let Some(wide) = wide {
                        quote! { (((a as #wide) * (b as #wide)) % (#n as #wide)) as #repr }
                    } else {
                        mul_expr(
                            repr,
                            modulus,
                            power_of_two,
                            quote!(a),
                            quote!(Self::#reduction(b)),
                            primitive.residue_bound(modulus),
                        )
                    }
                }
                PrimitiveKind::Signed(_) => {
                    if let Some(wide) = wide_signed.clone() {
                        quote! {
                            (((a as #wide) * (b as #wide)).rem_euclid(#n as #wide)) as #repr
                        }
                    } else {
                        mul_expr(
                            repr,
                            modulus,
                            power_of_two,
                            quote!(a),
                            quote!(Self::#reduction(b)),
                            primitive.residue_bound(modulus),
                        )
                    }
                }
            }
        } else {
            mul_expr(
                repr,
                modulus,
                power_of_two,
                quote!(a),
                quote!(Self::#reduction(b)),
                primitive.residue_bound(modulus),
            )
        };

        quote! {
            #[inline(always)]
            fn #add_name(a: #repr, b: #ty) -> #repr {
                #add_body
            }

            #[inline(always)]
            fn #mul_name(a: #repr, b: #ty) -> #repr {
                #mul_body
            }
        }
    });

    tokens.extend(quote! {
        const impl #type_name {
            #[inline(always)]
            fn __cyclique_add_mod(a: #repr, b: #repr) -> #repr {
                #add_mod
            }

            #[inline(always)]
            fn __cyclique_sub_mod(a: #repr, b: #repr) -> #repr {
                #sub_mod
            }

            #[inline(always)]
            fn __cyclique_neg_mod(a: #repr) -> #repr {
                #neg_mod
            }

            #[inline(always)]
            fn __cyclique_mul_mod(a: #repr, b: #repr) -> #repr {
                #mul_mod
            }

            #small_mul
            #(#reductions)*
            #(#primitive_ops)*
        }
    });

    let add_body = quote! { unsafe { Self::new_unchecked(Self::__cyclique_add_mod(*self, *rhs)) } };
    let sub_body = quote! { unsafe { Self::new_unchecked(Self::__cyclique_sub_mod(*self, *rhs)) } };
    let mul_body = quote! { unsafe { Self::new_unchecked(Self::__cyclique_mul_mod(*self, *rhs)) } };
    let neg_body = quote! { unsafe { Self::new_unchecked(Self::__cyclique_neg_mod(*self)) } };

    tokens.extend(quote! {
        const impl #ops::Add for #type_name {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self {
                #add_body
            }
        }

        const impl #ops::AddAssign for #type_name {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        const impl #ops::Sub for #type_name {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self {
                #sub_body
            }
        }

        const impl #ops::SubAssign for #type_name {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        const impl #ops::Mul for #type_name {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self {
                #mul_body
            }
        }

        const impl #ops::MulAssign for #type_name {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs;
            }
        }

        const impl #ops::Neg for #type_name {
            type Output = Self;
            #[inline(always)]
            fn neg(self) -> Self {
                #neg_body
            }
        }
    });

    for primitive in PRIMITIVES {
        let ty = primitive.ty();
        let reduction = reduction_name(&primitive);
        let add_name = format_ident!("__cyclique_add_primitive_{}", primitive.name);
        let mul_name = format_ident!("__cyclique_mul_primitive_{}", primitive.name);

        let reduced = quote! { Self::#reduction(rhs) };
        let add_expr = quote! {
            unsafe { Self::new_unchecked(Self::#add_name(*self, rhs)) }
        };
        let sub_expr = quote! {
            unsafe { Self::new_unchecked(Self::__cyclique_sub_mod(*self, #reduced)) }
        };
        let product_expr = quote! {
            unsafe { Self::new_unchecked(Self::#mul_name(*self, rhs)) }
        };

        let from_expr = quote! { unsafe { Self::new_unchecked(Self::#reduction(value)) } };

        let nonzero_mul = quote! {
            unsafe { Self::new_unchecked(Self::#mul_name(*self, rhs.get())) }
        };

        tokens.extend(quote! {
            const impl #ops::Add<#ty> for #type_name {
                type Output = Self;
                #[inline(always)]
                fn add(self, rhs: #ty) -> Self {
                    #add_expr
                }
            }

            const impl #ops::AddAssign<#ty> for #type_name {
                #[inline(always)]
                fn add_assign(&mut self, rhs: #ty) {
                    *self = *self + rhs;
                }
            }

            const impl #ops::Sub<#ty> for #type_name {
                type Output = Self;
                #[inline(always)]
                fn sub(self, rhs: #ty) -> Self {
                    #sub_expr
                }
            }

            const impl #ops::SubAssign<#ty> for #type_name {
                #[inline(always)]
                fn sub_assign(&mut self, rhs: #ty) {
                    *self = *self - rhs;
                }
            }

            const impl #ops::Mul<#ty> for #type_name {
                type Output = Self;
                #[inline(always)]
                fn mul(self, rhs: #ty) -> Self {
                    #product_expr
                }
            }

            const impl #ops::MulAssign<#ty> for #type_name {
                #[inline(always)]
                fn mul_assign(&mut self, rhs: #ty) {
                    *self = *self * rhs;
                }
            }

            const impl ::cyclique::__private::core::convert::From<#ty> for #type_name {
                #[inline(always)]
                fn from(value: #ty) -> Self {
                    #from_expr
                }
            }

            const impl ::cyclique::__private::core::convert::From<#num::NonZero<#ty>> for #type_name {
                #[inline(always)]
                fn from(value: #num::NonZero<#ty>) -> Self {
                    Self::from(value.get())
                }
            }

            const impl #ops::Mul<#num::NonZero<#ty>> for #type_name {
                type Output = Self;
                #[inline(always)]
                fn mul(self, rhs: #num::NonZero<#ty>) -> Self {
                    #nonzero_mul
                }
            }
        });
    }
}

fn add_mod_body(repr: ReprType, modulus: u128, power_of_two: bool) -> pm2::TokenStream {
    let n = repr_literal(repr, modulus);
    let mask = repr_literal(repr, modulus - 1);
    if power_of_two {
        quote! { a.wrapping_add(b) & #mask }
    } else if modulus <= (1u128 << (repr.bits() - 1)) {
        quote! {
            let sum = a + b;
            if sum >= #n { sum - #n } else { sum }
        }
    } else {
        quote! {
            let (sum, carry) = a.overflowing_add(b);
            let (wrapped, borrow) = sum.overflowing_sub(#n);
            if carry | !borrow { wrapped } else { sum }
        }
    }
}

fn sub_mod_body(repr: ReprType, modulus: u128, power_of_two: bool) -> pm2::TokenStream {
    let n = repr_literal(repr, modulus);
    if power_of_two {
        let mask = repr_literal(repr, modulus - 1);
        quote! { a.wrapping_sub(b) & #mask }
    } else {
        quote! {
            let (difference, borrow) = a.overflowing_sub(b);
            if borrow { difference.wrapping_add(#n) } else { difference }
        }
    }
}

fn neg_mod_body(repr: ReprType, modulus: u128, power_of_two: bool) -> pm2::TokenStream {
    let n = repr_literal(repr, modulus);
    let mask = repr_literal(repr, modulus - 1);
    if power_of_two {
        quote! { a.wrapping_neg() & #mask }
    } else {
        quote! {
            if a == 0 { 0 } else { #n - a }
        }
    }
}

fn mul_mod_body(repr: ReprType, modulus: u128) -> pm2::TokenStream {
    let n = repr_literal(repr, modulus);
    let mask = repr_literal(repr, modulus - 1);
    if modulus.is_power_of_two() {
        return quote! { a.wrapping_mul(b) & #mask };
    }
    if product_fits(repr, modulus, modulus - 1) {
        return quote! { a.wrapping_mul(b) % #n };
    }
    if let Some(wide) = repr.next() {
        return quote! {
            (((a as #wide) * (b as #wide)) % (#n as #wide)) as #repr
        };
    }
    if modulus <= (1u128 << 64) {
        return quote! { a.wrapping_mul(b) % #n };
    }
    montgomery_body(modulus)
}

fn montgomery_body(modulus: u128) -> pm2::TokenStream {
    match montgomery_constants(modulus) {
        Montgomery::Odd { nprime, r2 } => {
            let n = Literal::u128_suffixed(modulus);
            let nprime = Literal::u128_suffixed(nprime);
            let r2 = Literal::u128_suffixed(r2);
            quote! {
                let product = ::cyclique::__private::redc_mul(a, b, #n, #nprime);
                ::cyclique::__private::redc_mul(product, #r2, #n, #nprime)
            }
        }
        Montgomery::Even {
            odd,
            mask,
            nprime,
            r2,
            odd_inverse,
        } => {
            let odd = Literal::u128_suffixed(odd);
            let mask = Literal::u128_suffixed(mask);
            let nprime = Literal::u128_suffixed(nprime);
            let r2 = Literal::u128_suffixed(r2);
            let odd_inverse = Literal::u128_suffixed(odd_inverse);
            quote! {
                let low = a.wrapping_mul(b) & #mask;
                let a_mont = ::cyclique::__private::redc_mul(a, #r2, #odd, #nprime);
                let b_mont = ::cyclique::__private::redc_mul(b, #r2, #odd, #nprime);
                let product =
                    ::cyclique::__private::redc_mul(a_mont, b_mont, #odd, #nprime);
                let residue = ::cyclique::__private::redc(product, 0, #odd, #nprime);
                let factor = (low.wrapping_sub(residue) & #mask)
                    .wrapping_mul(#odd_inverse)
                    & #mask;
                residue.wrapping_add(factor.wrapping_mul(#odd))
            }
        }
    }
}

/// Whether `a * b` stays within `repr` for `a < modulus` and `b <= bound`.
fn product_fits(repr: ReprType, modulus: u128, bound: u128) -> bool {
    bound <= repr.max_value() / (modulus - 1)
}

fn mul_expr(
    repr: ReprType,
    modulus: u128,
    power_of_two: bool,
    a: pm2::TokenStream,
    b: pm2::TokenStream,
    bound: u128,
) -> pm2::TokenStream {
    if !power_of_two && product_fits(repr, modulus, bound) {
        quote! { Self::__cyclique_mul_mod_small(#a, #b) }
    } else {
        quote! { Self::__cyclique_mul_mod(#a, #b) }
    }
}

fn reduce_expr(
    repr: ReprType,
    modulus: u128,
    power_of_two: bool,
    primitive: Primitive,
    value: pm2::TokenStream,
) -> pm2::TokenStream {
    let n = repr_literal(repr, modulus);
    if power_of_two {
        let mask = repr_literal(repr, modulus - 1);
        return quote! { ((#value) as #repr) & #mask };
    }
    let ty = primitive.ty();
    match primitive.kind {
        PrimitiveKind::Unsigned(bits) => {
            if let Some(bits) = bits {
                if modulus > max_unsigned(bits) {
                    quote! { (#value) as #repr }
                } else {
                    quote! { (((#value) % (#n as #ty)) as #repr) }
                }
            } else {
                let modulus_u128 = Literal::u128_suffixed(modulus);
                quote! {
                    if #modulus_u128 <= <#ty>::MAX as u128 {
                        (((#value) % (#n as #ty)) as #repr)
                    } else {
                        (#value) as #repr
                    }
                }
            }
        }
        PrimitiveKind::Signed(bits) => {
            if let Some(bits) = bits {
                if modulus > max_signed(bits) {
                    quote! {
                        if (#value) < 0 {
                            (#n).wrapping_add((#value) as #repr)
                        } else {
                            (#value) as #repr
                        }
                    }
                } else {
                    quote! { (((#value).rem_euclid(#n as #ty)) as #repr) }
                }
            } else {
                let modulus_u128 = Literal::u128_suffixed(modulus);
                quote! {
                    if #modulus_u128 <= <#ty>::MAX as u128 {
                        ((#value).rem_euclid(#n as #ty)) as #repr
                    } else if (#value) < 0 {
                        (#n).wrapping_add((#value) as #repr)
                    } else {
                        (#value) as #repr
                    }
                }
            }
        }
    }
}
