use std::iter::{self, once};

use proc_macro2 as pm2;
use quote::{format_ident, quote};

const PITCH_NAMES: [&str; 17] = [
    "C", "C sharp", "D flat", "D", "D sharp", "E flat", "E", "F", "F sharp", "G flat", "G",
    "G sharp", "A flat", "A", "A sharp", "B flat", "B",
];
const STEP_NAMES: [&[&str]; 7] = [
    &["C", "do", "ut"],
    &["D", "re"],
    &["E", "mi"],
    &["F", "fa"],
    &["G", "sol"],
    &["A", "la"],
    &["B", "si", "ti"],
];
const INTERVAL_NAMES: [&str; 7] = [
    "unison", "second", "third", "fourth", "fifth", "sixth", "seventh",
];
const NEIGHBOURING_WHITE_KEYS: [i8; 5] = [0, 1, 3, 4, 5];
const ACCIDENTALS: [(&str, i8); 7] = [
    ("triple sharp", 3),
    ("double sharp", 2),
    ("sharp", 1),
    ("natural", 0),
    ("flat", -1),
    ("double flat", -2),
    ("triple flat", -3),
];

fn opitch_var_name(pitch_name: &str) -> syn::Ident {
    syn::Ident::new(
        &pitch_name.to_uppercase().replace(' ', "_"),
        pm2::Span::call_site(),
    )
}

fn pitch_var_name(pitch_name: &str, octave: i8) -> syn::Ident {
    syn::Ident::new(
        &format!(
            "{}_{}",
            pitch_name.to_uppercase().replace(' ', "_"),
            // replace minus sign with `M` to make it a valid identifier
            octave.to_string().replace('-', "M")
        ),
        pm2::Span::call_site(),
    )
}

fn uncased_str_path() -> pm2::TokenStream {
    quote!(uncased::UncasedStr)
}

fn phf_map_path() -> pm2::TokenStream {
    quote!(phf::phf_map)
}

pub fn gen_step_name_to_step_table() -> pm2::TokenStream {
    let mut map_entries = quote!();

    let phf_map_path = phf_map_path();
    let uncased_str_path = uncased_str_path();
    let ostep_path = ostep_path();

    for (step, &step_names) in STEP_NAMES.iter().enumerate() {
        let step = step as i8;
        let num_step_name = (step + 1).to_string();
        let variant_name = format_ident!("{}", step_names[0]);
        map_entries.extend(
            step_names
                .iter()
                .copied()
                .chain(iter::once(num_step_name.as_str()))
                .map(|v| quote!(#uncased_str_path::new(#v))) // convert to uncased
                .intersperse(quote!(|)),
        );
        map_entries.extend(quote!(=> #ostep_path::#variant_name,));
    }
    quote! {
        #phf_map_path! {
            #map_entries
        }
    }
}

pub fn gen_ostep_consts() -> pm2::TokenStream {
    let mut const_decls = quote!();
    let mut variants = quote!();
    let ostep_path = ostep_path();
    for (&step_names, interval_name) in STEP_NAMES.iter().zip(INTERVAL_NAMES) {
        let doc_string = format!(
            "{}, or interval degree **{}**",
            step_names
                .iter()
                .map(|v| format!("**{v}**"))
                .collect::<Vec<_>>()
                .join(" / "),
            interval_name
        );
        let variant_name = format_ident!("{}", step_names[0]);
        variants.extend(quote! {
            #[doc = #doc_string]
            #variant_name,
        });
        for step_name in step_names[1..].iter().copied().chain(once(interval_name)) {
            let step_name = format_ident!("{}", step_name.to_uppercase());
            const_decls.extend(quote! {
                #[doc = #doc_string]
                pub const #step_name: Self = Self::#variant_name;
            });
        }
    }
    quote! {
        /// An octave-unspecific **step**, i.e. note name without accidental.
        ///
        /// Is one of: **C**, **D**, **E**, **F**, **G**, **A**, **B**, represented by integer
        /// 0 to 6 respectively. Addition, subtraction and scalar multiplication of step follows the
        /// arithmetic of [quotient group](https://en.wikipedia.org/wiki/Quotient_group) $ZZ_7$.
        #[derive(Copy, Hash, Debug, Display)]
        #[derive_const(Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
        #[cfg_attr(
            feature = "rkyv",
            derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
        )]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub enum OStep {
            #[default]
            #variants
        }
        impl #ostep_path {
            #const_decls
        }
    }
}

pub fn gen_step_consts(octave_range: impl IntoIterator<Item = i8>) -> pm2::TokenStream {
    let mut const_decls = quote!();
    let ostep_path = ostep_path();
    let step_path = step_path();
    for octave in octave_range {
        for step_names in STEP_NAMES {
            let doc_string = format!(
                "{} in octave {octave}",
                step_names
                    .iter()
                    .map(|v| format!("**{v}**"))
                    .collect::<Vec<_>>()
                    .join(" / "),
            );
            for step_name in step_names.iter().copied() {
                let step_name = format_ident!(
                    "{}_{}",
                    step_name.to_uppercase(),
                    octave.to_string().replace('-', "M")
                );
                let variant_name = format_ident!("{}", step_names[0]);
                const_decls.extend(quote! {
                    #[doc = #doc_string]
                    pub const #step_name: Self = #ostep_path::#variant_name.put_in_octave(#octave);
                });
            }
        }
    }
    quote! {
        impl #step_path {
            #const_decls
        }
    }
}

pub fn gen_otone_consts() -> pm2::TokenStream {
    let mut const_decls = quote!();
    let opitch_path = opitch_path();
    let otone_path = otone_path();

    for pitch_name in PITCH_NAMES {
        let var_name = opitch_var_name(pitch_name);
        let doc_string = format!("Tone corresponding to pitch {pitch_name}.");
        const_decls.extend(quote! {
            #[doc = #doc_string]
            pub const #var_name: Self = #opitch_path::#var_name.tone.generalize();
        });
    }

    quote! {
        impl #otone_path {
            #const_decls
        }
    }
}

pub fn gen_tone_consts(octave_range: impl Iterator<Item = i8>) -> pm2::TokenStream {
    let mut const_decls = quote!();
    let pitch_path = pitch_path();
    let tone_path = tone_path();
    for octave in octave_range {
        for pitch_name in PITCH_NAMES {
            let var_name = pitch_var_name(pitch_name, octave);
            let doc_string = format!("Tone corresponding to pitch {pitch_name} in octave {octave}");
            const_decls.extend(quote! {
                #[doc = #doc_string]
                pub const #var_name: Self = #pitch_path::#var_name.tone;
            });
        }
    }
    quote! {
        impl #tone_path {
            #const_decls
        }
    }
}

pub fn gen_opitch_consts() -> pm2::TokenStream {
    let mut const_decls = quote!();

    let opitch_path = opitch_path();
    let ostep_path = ostep_path();
    let acci_path = acci_path();

    for pitch_name in PITCH_NAMES {
        let var_name = opitch_var_name(pitch_name);
        let const_def = if let Some((step_name, acci_name)) = pitch_name.rsplit_once(" ") {
            let step_name = syn::Ident::new(step_name, pm2::Span::call_site());
            let acci_name = syn::Ident::new(&acci_name.to_uppercase(), pm2::Span::call_site());
            quote! {
                #opitch_path::diatonic(#ostep_path::#step_name) + #acci_path::#acci_name
            }
        } else {
            quote! {
                #opitch_path::diatonic(#ostep_path::#var_name)
            }
        };
        const_decls.extend(quote! {
            #[doc = #pitch_name]
            pub const #var_name: Self = #const_def;
        });
    }
    quote! {
        impl #opitch_path {
            #const_decls
        }
    }
}

pub fn gen_pitch_consts(octave_range: impl Iterator<Item = i8>) -> pm2::TokenStream {
    let mut const_decls = quote!();
    let opitch_path = opitch_path();
    let pitch_path = pitch_path();
    for octave in octave_range {
        for pitch_name in PITCH_NAMES {
            let doc_str = format!("{pitch_name} in octave {octave}");
            let opitch_var_name = syn::Ident::new(
                &pitch_name.to_uppercase().replace(' ', "_"),
                pm2::Span::call_site(),
            );
            let var_name = pitch_var_name(pitch_name, octave);
            const_decls.extend(quote! {
                #[doc = #doc_str]
                pub const #var_name: Self = (#opitch_path::#opitch_var_name).put_in_octave(#octave);
            });
        }
    }
    quote! {
        impl #pitch_path {
            #const_decls
        }
    }
}

pub fn gen_ointerval_consts() -> pm2::TokenStream {
    let mut const_decls = quote!();
    let ointerval_path = ointerval_path();
    let interval_qual_path = interval_qual_path();
    let ostep_path = ostep_path();
    for (i, &interval_name) in INTERVAL_NAMES.iter().enumerate() {
        let quals: &[&str] = match i {
            0 | 3 | 4 => &["diminished", "perfect", "augmented"],
            _ => &["diminished", "minor", "major", "augmented"],
        };
        let interval_ident =
            syn::Ident::new(&interval_name.to_ascii_uppercase(), pm2::Span::call_site());
        for qual in quals {
            let doc_string = format!("**{} {}**", qual.word_to_titlecase(), interval_name);
            let var_name = syn::Ident::new(
                &format!(
                    "{}_{}",
                    qual.to_ascii_uppercase(),
                    interval_name.to_ascii_uppercase()
                ),
                pm2::Span::call_site(),
            );
            let qual_ident = syn::Ident::new(&qual.to_uppercase(), pm2::Span::call_site());
            const_decls.extend(quote! {
                #[doc = #doc_string]
                pub const #var_name: Self = Self {
                    step: #ostep_path::#interval_ident,
                    qual: #interval_qual_path::#qual_ident,
                };
            });
        }
    }
    quote! {
        impl #ointerval_path {
            #const_decls
        }
    }
}

pub fn gen_black_key_struct() -> pm2::TokenStream {
    let black_key_path = black_key_path();
    let mut variants = quote!();
    let mut const_decls = quote!();
    for (i, &tone) in NEIGHBOURING_WHITE_KEYS.iter().enumerate() {
        let sharp_step_name = STEP_NAMES[tone as usize][0];
        let flat_step_name = STEP_NAMES[tone as usize + 1][0];
        let variant_name = format_ident!("{sharp_step_name}Sharp{flat_step_name}Flat");
        let sharp_var_name = format_ident!("{sharp_step_name}_SHARP");
        let flat_var_name = format_ident!("{flat_step_name}_FLAT");
        let variant_doc =
            format!("Black key {i}: **{sharp_step_name} sharp**, or **{flat_step_name} flat**.");
        variants.extend(quote! {
            #[doc = #variant_doc]
            #variant_name,
        });
        const_decls.extend(quote! {
            #[doc = #variant_doc]
            pub const #sharp_var_name: Self = Self::#variant_name;
            #[doc = #variant_doc]
            pub const #flat_var_name: Self = Self::#variant_name;
        });
    }
    quote! {
        /// Indication of all black keys in an octave on the piano keyboard.
        #[derive(Debug, Copy, Hash, ::enum_ordinalize::Ordinalize)]
        #[derive_const(Clone, PartialEq, Eq, PartialOrd, Ord)]
        #[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
        #[cfg_attr(feature = "rkyv", derive(::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize))]
        pub enum BlackKey {
            #variants
        }

        impl #black_key_path {
            #const_decls
        }
    }
}

pub fn gen_acci_pref_methods() -> pm2::TokenStream {
    let acci_pref_path = acci_pref_path();
    let black_key_path = black_key_path();
    let mut method_decls = quote!();
    for tone in NEIGHBOURING_WHITE_KEYS {
        let sharp_step_name = STEP_NAMES[tone as usize][0];
        let flat_step_name = STEP_NAMES[tone as usize + 1][0];
        let variant_name = format_ident!("{sharp_step_name}Sharp{flat_step_name}Flat");

        let sharp_name_doc = format!(
            "Whether **{sharp_step_name} sharp** is preferred over **{flat_step_name} flat**."
        );
        let flat_name_doc = format!(
            "Whether **{flat_step_name} flat** is preferred over **{sharp_step_name} sharp**."
        );
        let with_sharp_name_doc =
            format!("Prefers **{sharp_step_name} sharp** over **{flat_step_name} flat**.");
        let with_flat_name_doc =
            format!("Prefers **{flat_step_name} flat** over **{sharp_step_name} sharp**.");

        let sharp_name = format_ident!("{}_sharp", sharp_step_name.to_lowercase());
        let flat_name = format_ident!("{}_flat", flat_step_name.to_lowercase());
        let with_sharp_name = format_ident!("with_{}_sharp", sharp_step_name.to_lowercase());
        let with_flat_name = format_ident!("with_{}_flat", flat_step_name.to_lowercase());
        method_decls.extend(quote! {
            #[inline(always)]
            #[doc = #sharp_name_doc]
            pub const fn #sharp_name(&self) -> bool {
                self.is_sharp(#black_key_path::#variant_name)
            }

            #[inline(always)]
            #[doc = #flat_name_doc]
            pub const fn #flat_name(&self) -> bool {
                self.is_flat(#black_key_path::#variant_name)
            }

            #[inline(always)]
            #[doc = #with_sharp_name_doc]
            pub const fn #with_sharp_name(self) -> Self {
                self.with_sharp(#black_key_path::#variant_name)
            }

            #[inline(always)]
            #[doc = #with_flat_name_doc]
            pub const fn #with_flat_name(self) -> Self {
                self.with_flat(#black_key_path::#variant_name)
            }
        });
    }
    quote! {
        impl #acci_pref_path {
            #method_decls
        }
    }
}

pub fn gen_acci_ops_trait() -> pm2::TokenStream {
    let mut methods = quote!();
    let acci_path = acci_path();
    for (acci_name, value) in ACCIDENTALS {
        if value == 0 {
            continue;
        }
        let method_name = format_ident!("{}", acci_name.replace(' ', "_"));
        let acci_var_name = format_ident!("{}", acci_name.replace(' ', "_").to_ascii_uppercase());
        let doc_string = format!("Modifies the current object by accidental **{acci_name}**.");
        methods.extend(quote! {
            #[doc = #doc_string]
            fn #method_name(mut self) -> Self {
                self += #acci_path::#acci_var_name;
                self
            }
        });
    }
    quote! {
        /// Helper trait that provides methods for modifying pitch with accidentals.
        pub impl(crate) trait AcciOps: ::std::ops::AddAssign<#acci_path> + Sized {
            #methods
        }
        impl<T: ::std::ops::AddAssign<#acci_path>> AcciOps for T {}
    }
}

// MARK: paths

fn ostep_path() -> pm2::TokenStream {
    quote!(crate::step::OStep)
}

fn step_path() -> pm2::TokenStream {
    quote!(crate::step::Step)
}

fn acci_path() -> pm2::TokenStream {
    quote!(crate::acci::Acci)
}

fn opitch_path() -> pm2::TokenStream {
    quote!(crate::pitch::OPitch)
}

fn pitch_path() -> pm2::TokenStream {
    quote!(crate::pitch::Pitch)
}

fn otone_path() -> pm2::TokenStream {
    quote!(crate::tone::OTone)
}

fn tone_path() -> pm2::TokenStream {
    quote!(crate::tone::Tone)
}

fn interval_qual_path() -> pm2::TokenStream {
    quote!(crate::interval::IntervalQual)
}

fn ointerval_path() -> pm2::TokenStream {
    quote!(crate::interval::OInterval)
}

fn acci_pref_path() -> pm2::TokenStream {
    quote!(crate::acci_pref::AcciPref)
}

fn black_key_path() -> pm2::TokenStream {
    quote!(crate::acci_pref::BlackKey)
}
