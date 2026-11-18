use proc_macro2::{self as pm2};
use quote::quote;
use syn::Ident;

use crate::{
    Divisor, ReprType, ReprTypeOrZST,
    repr::NonBoundaryDivisor,
    utils::{mem_path, num_path, ops_path},
};

// macro apply_with_nonzero {
//     () => {},
//     ($macro:ident; $($from:ty),+$(,)?) => {
//         $macro!($($from),+);
//         $macro!($(::cyclique::__private::core::num::NonZero<$from>),+);
//     }
// }

pub fn expand_new_unchecked_impl(
    tokens: &mut pm2::TokenStream,
    type_name: &Ident,
    count: NonBoundaryDivisor,
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
            unsafe fn new_unchecked(value: #repr_type) -> Self {
                unsafe { #mem::transmute(value) }
            }
        }
    });
}

pub fn expand_ops_impls(tokens: &mut pm2::TokenStream, type_name: &Ident, count: Divisor) {
    let repr_type = count.repr_type();
    let mem = mem_path();
    if repr_type != ReprTypeOrZST::ZST {
        tokens.extend(quote! {
            const impl ::cyclique::__private::core::ops::Deref for #type_name {
                type Target = #repr_type;
                fn deref(&self) -> &Self::Target {
                    // SAFETY: the modulus type has the same memory layout as its
                    // representation type
                    unsafe { #mem::transmute(self) }
                }
            }
        });
    }

    if let Ok(count) = NonBoundaryDivisor::try_from(count) {
        expand_ops_impls_non_boundary(tokens, type_name, count);
    } else {
        expand_ops_impls_boundary(tokens, type_name, count.repr_type());
    }
}

fn expand_ops_impls_non_boundary(
    tokens: &mut pm2::TokenStream,
    type_name: &Ident,
    count: NonBoundaryDivisor,
) {
    let ops = ops_path();
    let repr_type = count.repr_type();

    tokens.extend(quote! {
        const impl #ops::Add for #type_name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                let (mut result, overflow) = (*self).overflowing_add(*rhs);
                // SAFETY: have checked that the resulting value fits in the range
                unsafe {
                    Self::new_unchecked(if overflow || result >= #count {
                        result.wrapping_sub(#count)
                    } else {
                        result
                    })
                }
            }
        }

        const impl #ops::AddAssign for #type_name {
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs
            }
        }

        const impl #ops::Sub for #type_name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                // SAFETY: have checked that the resulting value fits in the range
                unsafe {
                    Self::new_unchecked(if *self >= *rhs {
                        (*self).wrapping_sub(*rhs)
                    } else {
                        (*self).wrapping_sub(*rhs).wrapping_add(#count)
                    })
                }
            }
        }

        const impl #ops::Neg for #type_name {
            type Output = Self;
            fn neg(self) -> Self {
                // SAFETY: have checked that the resulting value fits in the range
                unsafe {
                    Self::new_unchecked(if *self == 0 {
                        0
                    } else {
                        #count - *self
                    })
                }
            }
        }
    });

    for rhs in ReprType::ALL.iter().copied() {
        let rhs_signed = rhs.to_signed();
        if let Some(wider_type) = repr_type.next() {
            let mul_impl = if repr_type <= rhs {
                quote! {
                    unsafe {
                        Self::new_unchecked(
                            (*self as #wider_type)
                                .wrapping_mul(rhs as #wider_type)
                                .rem_euclid(#count as #wider_type)
                            as #repr_type
                        )
                    }
                }
            } else {
                quote! {
                    unsafe {
                        Self::new_unchecked(
                            (*self as #wider_type)
                                // take modulus for `rhs` to make it fit in range of `repr_type`
                                .wrapping_mul(rhs.rem_euclid(#count as #rhs) as #wider_type)
                                // do mul-mod operation in the type wider than `repr_type`
                                .rem_euclid(#count as #wider_type)
                            as #repr_type
                        )
                    }
                }
            };
            tokens.extend(quote! {
                const impl #ops::Mul<#rhs> for #type_name {
                    type Output = Self;
                    fn mul(self, rhs: #rhs) -> Self {
                        #mul_impl
                    }
                }
            });
        } else {
        }
    }
}

fn expand_impls_zst(tokens: &mut pm2::TokenStream, type_name: &Ident) {
    let ops = ops_path();
    let mem = mem_path();
    let unique_value = quote! {
        // SAFETY: transmutation of ZST
        unsafe { #mem::transmute(()) }
    };

    // tokens.extend(quote! {
    //     const impl #ops::Add for #type_name {
    //         type Output = Self;
    //         fn add(self, _rhs: Self) -> Self { #unique_value }
    //     }

    //     const impl #ops::AddAssign for #type_name {
    //         fn add_assign(&mut self, rhs: Self) {} // does nothing
    //     }

    //     const impl #ops::Sub for #type_name {
    //         type Output = Self;
    //         fn sub(self, _rhs: Self) -> Self { #unique_value }
    //     }

    //     const impl #ops::SubAssign for #type_name {
    //         fn sub_assign(&mut self, rhs: Self) {} // does nothing
    //     }

    //     const impl #ops::Neg for #type_name {
    //         type Output = Self;
    //         fn neg(self) -> Self { #unique_value }
    //     }
    // });

    // macro impl_primitive_conv_ops {
    //     () => {},
    //     ($($from:ty),+$(,)?) => {
    //         $(tokens.extend(quote!{
    //             const impl From<$from> for #type_name {
    //                 fn from(_value: $from) -> Self { #unique_value }
    //             }
    //             const impl #ops::Add<$rhs> for #type_name {
    //                 type Output = Self;
    //                 fn add(self, _rhs: $rhs) -> Self { #unique_value }
    //             }
    //             const impl #ops::AddAssign<$rhs> for #type_name {
    //                 fn add_assign(&mut self, _rhs: $rhs) { } // does nothing
    //             }
    //             const impl #ops::Mul<$rhs> for #type_name {
    //                 type Output = Self;
    //                 fn mul(self, _rhs: $rhs) -> Self { #unique_value }
    //             }
    //             const impl #ops::MulAssign<$rhs> for #type_name {
    //                 fn mul_assign(&mut self, _rhs: $rhs) { } // does nothing
    //             }
    //         });)+
    //     }
    // }

    // apply_with_nonzero!(
    //     impl_primitive_conv_ops;
    //     u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize
    // );
}

fn expand_ops_impls_boundary(
    tokens: &mut pm2::TokenStream,
    type_name: &Ident,
    repr_type: ReprTypeOrZST,
) {
    if repr_type == ReprTypeOrZST::ZST {
        expand_impls_zst(tokens, type_name);
        return;
    }

    let ops = ops_path();
    let num = num_path();

    tokens.extend(quote! {
        impl ::cyclique::ModulusType for #type_name {
            type Repr = #repr_type;
            const DIVISOR: #repr_type = 0;
        }

        const impl #ops::Add for #type_name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self::new((*self).wrapping_add(*rhs))
            }
        }

        const impl #ops::AddAssign for #type_name {
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs
            }
        }

        const impl #ops::Sub for #type_name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                // SAFETY: have checked that the resulting value fits in the range
                Self::new((*self).wrapping_sub(*rhs))
            }
        }

        const impl #ops::Neg for #type_name {
            type Output = Self;
            fn neg(self) -> Self {
                // SAFETY: have checked that the resulting value fits in the range
                Self::new((*self).wrapping_neg())
            }
        }
    });

    macro impl_conv {
        () => {},
        ($($from:ty),+$(,)?) => {
            $(tokens.extend(quote!{
                const impl From<$from> for #type_name {
                    fn from(value: $from) -> Self {
                        Self::new(value as #repr_type)
                    }
                }
                const impl From<#num::NonZero<$from>> for #type_name {
                    fn from(value: #num::NonZero<$from>) -> Self {
                        Self::new(value.get() as #repr_type)
                    }
                }
            });)+
        }
    }

    macro impl_mul {
        () => {},
        ($($rhs:ty),+$(,)?) => {
            $(tokens.extend(quote!{
                const impl #ops::Mul<$rhs> for #type_name {
                    type Output = Self;
                    fn mul(self, rhs: $rhs) -> Self {
                        Self::new((*self).wrapping_mul(rhs as #repr_type))
                    }
                }
                const impl #ops::Mul<#num::NonZero<$rhs>> for #type_name {
                    type Output = Self;
                    fn mul(self, rhs: #num::NonZero<$rhs>) -> Self {
                        Self::new((*self).wrapping_mul(rhs.get() as #repr_type))
                    }
                }
            });)+
        }
    }

    impl_conv!(
        u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
    );
    impl_mul!(
        u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
    );
}
