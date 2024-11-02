extern crate proc_macro;

// use darling::FromMeta;
// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Ident};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Lit, Meta, Path};

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
