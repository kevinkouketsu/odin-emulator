extern crate proc_macro;

// use darling::FromMeta;
// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Ident};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Expr, Lit, Meta, Path};

#[proc_macro_derive(MessageSignalDerive, attributes(identifier))]
pub fn writable_resource_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;
    let mut identifier = None;
    for attr in input.attrs {
        if attr.path().is_ident("identifier") {
            if let Meta::NameValue(meta_name_value) = attr.meta {
                if let Expr::Lit(e) = meta_name_value.value {
                    if let Lit::Str(lit_str) = e.lit {
                        if let Ok(path) = lit_str.parse::<Path>() {
                            identifier = Some(path);
                        }
                    }
                }
            }
        }
    }

    let identifier = match identifier {
        Some(id) => id,
        None => {
            return syn::Error::new_spanned(
                struct_name,
                "Expected #[identifier = \"ServerMessage::Variant\"]",
            )
            .to_compile_error()
            .into();
        }
    };

    let expanded = quote! {
        impl WritableResource for #struct_name {
            const IDENTIFIER: ServerMessage = #identifier;
            type Output = MessageSignal<#struct_name>;

            fn write(self) -> Result<Self::Output, WritableResourceError> {
                Ok(MessageSignal::default())
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(HandlerDerive, attributes(raw))]
pub fn handler_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = if let Data::Enum(data_enum) = &input.data {
        &data_enum.variants
    } else {
        panic!("HandlerDerive can only be applied to enums");
    };

    // Generate match arms for each variant
    let match_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        // Find the `#[raw = "RawType"]` attribute
        let raw_type = variant
            .attrs
            .iter()
            .find_map(|attr| {
                if let Meta::NameValue(meta_name_value) = &attr.meta {
                    if meta_name_value.path.is_ident("raw") {
                        if let Expr::Lit(e) = &meta_name_value.value {
                            if let Lit::Str(lit_str) = &e.lit {
                                return Some(lit_str.value());
                            }
                        }
                    }
                }
                None
            })
            .expect("Each variant must have a #[raw = \"...\"] attribute");

        // Convert the raw_type string to an Ident for use in the generated code
        let raw_type_ident = syn::Ident::new(&raw_type, variant.span());

        // Generate the match arm
        quote! {
            ClientMessage::#variant_name => {
                Message::#variant_name(#raw_type_ident::from_bytes(rest)?.1.try_into()?)
            }
        }
    });

    // Generate the full `TryFrom` implementation
    let expanded = quote! {
        impl TryFrom<((&[u8], usize), Header)> for #name {
            type Error = MessageError;

            fn try_from((rest, header): ((&[u8], usize), Header)) -> Result<Self, Self::Error> {
                let message_type = ClientMessage::try_from(header.typ)
                    .map_err(|_| MessageError::NotRecognized(header.clone()))?;

                Ok(match message_type {
                    #(#match_arms),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
