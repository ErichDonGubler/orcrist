use {
    proc_macro2::{Ident, Span, TokenStream},
    quote::quote,
    syn::parse_quote,
    synstructure::{decl_derive, Structure},
};

fn from_fixed_bytes_derive(s: Structure) -> TokenStream {
    let derived_name = s.ast().ident.clone();
    let fields_enum_name = {
        let new_ident = Ident::new(
            &format!("__FIELDS_OF_{}", s.ast().ident.to_string()),
            Span::call_site(),
        );
        quote! { #new_ident }
    };
    let trait_name = quote! { ::orcrist::FromFixedBytes };
    let trait_fields_type = quote! { FieldEnum };

    let resolve_ident = |f_opt: Option<&Ident>, idx| {
        f_opt
            .cloned()
            .unwrap_or_else(|| Ident::new(&format!("Inner{}", idx), Span::call_site()))
    };

    let fields_enum = {
        let fields_enum: TokenStream = {
            let variants_of_fields_enum = s.variants().iter().map(|v| {
                let variants = v.bindings().iter().enumerate().map(|(idx, b)| {
                    let ident = resolve_ident(b.ast().ident.as_ref(), idx);
                    let type_ = &b.ast().ty;
                    quote! { #ident(<#type_ as #trait_name>::#trait_fields_type) }
                });

                quote! {
                    #(#variants,)*
                }
            });

            parse_quote! {
                #[derive(Debug)]
                #[allow(non_camel_case_types)]
                enum #fields_enum_name {
                    #(#variants_of_fields_enum)*
                }
            }
        };

        let fields_display_match_arms = s.variants().iter().map(|v| {
            v.bindings().iter().enumerate().map(|(idx, b)| {
                let ident_maybe = b.ast().ident.as_ref();
                let ident = resolve_ident(ident_maybe, idx);
                let msg = format!(
                    "{{}} of {}",
                    if let Some(ident) = b.ast().ident.as_ref() {
                        format!("`{}`", ident.to_string())
                    } else {
                        format!("position {}", idx)
                    }
                );
                quote! {
                    #fields_enum_name::#ident(inner) => w(format_args!(#msg, inner)),
                }
            })
        });

        quote! {
            #fields_enum

            impl ::std::fmt::Display for #fields_enum_name {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    let mut w = |args| write!(f, "{} field", args);
                    match self {
                        #(#(#fields_display_match_arms)*)*
                    }
                }
            }
        }
    };

    let stream_param_ident = quote! { stream };

    let derived_name_str = derived_name.to_string();
    let read_impls = {
        let impls = s.variants().iter().map(|v| {
            let field_reads = v.construct(|f, idx| {
                let field_name = resolve_ident(f.ident.as_ref(), idx);
                quote! {
                    #trait_name::from_fixed_bytes(#stream_param_ident).map_err(|e| e.map_field(#derived_name_str, #fields_enum_name::#field_name))?
                }
            });
            quote!{
                #(#field_reads)*
            }
        });

        quote! {
            #(#impls)*
        }
    };

    let trait_impl = s.bound_impl(trait_name, quote! {
        type FieldEnum = #fields_enum_name;

        fn from_fixed_bytes<R: ::std::io::Read>(#stream_param_ident: &mut R) -> Result<Self, ::orcrist::ByteReadFailure<Self::#trait_fields_type>> {
            Ok(#(#read_impls)*)
        }
    });

    quote! {
        #fields_enum

        #trait_impl
    }
}
decl_derive!([FromFixedBytes] => from_fixed_bytes_derive);

#[cfg(test)]
mod test {
    use {
        crate::from_fixed_bytes_derive,
        orcrist::{Be, Le},
        synstructure::test_derive,
    };

    #[test]
    fn struct_named_fields() {
        test_derive! {
            from_fixed_bytes_derive {
                struct Asdf {
                    meh: u8,
                    blarg: Be<u16>,
                }
            }
            expands to {
                #[derive(Debug)]
                #[allow(non_camel_case_types)]
                enum __FIELDS_OF_Asdf {
                    meh(<u8 as ::orcrist::FromFixedBytes>::FieldEnum),
                    blarg(<Be<u16> as ::orcrist::FromFixedBytes>::FieldEnum),
                }
                impl ::std::fmt::Display for __FIELDS_OF_Asdf {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        let mut w = |args| write!(f, "{} field", args);
                        match self {
                            __FIELDS_OF_Asdf::meh(inner) => w(format_args!(
                                "{} of meh",
                                inner
                            )),
                            __FIELDS_OF_Asdf::blarg(inner) => w(format_args!(
                                "{} of blarg",
                                inner
                            )),
                        }
                    }
                }
                #[allow(non_upper_case_globals)]
                const _DERIVE_orcrist_FromFixedBytes_FOR_Asdf: () = {
                    impl ::orcrist::FromFixedBytes for Asdf {
                        type FieldEnum = __FIELDS_OF_Asdf;
                        fn from_fixed_bytes<R: ::std::io::Read>(
                            stream: &mut R
                        ) -> Result<Self, ::orcrist::ByteReadFailure<Self::FieldEnum>> {
                            Ok(Asdf {
                                meh: ::orcrist::FromFixedBytes::from_fixed_bytes(stream)
                                    .map_err(|e| e.map_field("Asdf", __FIELDS_OF_Asdf::meh))?,
                                blarg: ::orcrist::FromFixedBytes::from_fixed_bytes(stream)
                                    .map_err(|e| e.map_field("Asdf", __FIELDS_OF_Asdf::blarg))?,
                            })
                        }
                    }
                };
            }
        }
    }

    #[test]
    fn struct_newtype() {
        test_derive! {
            from_fixed_bytes_derive {
                struct Wat(Le<u32>);
            }
            expands to {
                #[derive(Debug)]
                #[allow(non_camel_case_types)]
                enum __FIELDS_OF_Wat {
                    Inner0(<Le<u32> as ::orcrist::FromFixedBytes>::FieldEnum),
                }
                impl ::std::fmt::Display for __FIELDS_OF_Wat {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        let mut w = |args| write!(f, "{} field", args);
                        match self {
                            __FIELDS_OF_Wat::Inner0(inner) => w(format_args!("{} of Inner0", inner)),
                        }
                    }
                }
                #[allow(non_upper_case_globals)]
                const _DERIVE_orcrist_FromFixedBytes_FOR_Wat: () = {
                    impl ::orcrist::FromFixedBytes for Wat {
                        type FieldEnum = __FIELDS_OF_Wat;
                        fn from_fixed_bytes<R: ::std::io::Read>(
                            stream: &mut R
                        ) -> Result<Self, ::orcrist::ByteReadFailure<Self::FieldEnum>> {
                            Ok(Wat(::orcrist::FromFixedBytes::from_fixed_bytes(stream)
                                .map_err(|e| e.map_field("Wat", __FIELDS_OF_Wat::Inner0))?,))
                        }
                    }
                };
            }
        }
    }
}
