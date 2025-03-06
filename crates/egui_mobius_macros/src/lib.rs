use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

#[proc_macro_derive(EventMacro)]
pub fn event_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Ensure macro is applied only to enums
    let variants = if let Data::Enum(DataEnum { variants, .. }) = &input.data {
        variants
    } else {
        return TokenStream::from(quote! {
            compile_error!("EventMacro can only be derived for enums");
        });
    };

    let variant_idents: Vec<_> = variants.iter().map(|v| &v.ident).collect();

    // Generate code for event_name only once
    let expanded = quote! {
        impl #name {
            /// Returns the event name as a string.
            pub fn event_name(&self) -> &'static str {
                match self {
                    #(Self::#variant_idents { .. } => stringify!(#variant_idents),)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
