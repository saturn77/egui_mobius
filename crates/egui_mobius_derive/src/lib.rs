// crates/egui_mobius_derive/src/lib.rs
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Type};

#[proc_macro_derive(MobiusWidgetReactive)]
pub fn derive_mobius_widget_reactive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("MobiusWidgetReactive only supports named fields"),
        },
        _ => panic!("MobiusWidgetReactive can only be used on structs"),
    };

    let impls = [
        ("String", "String"),
        ("i32", "i32"),
        ("f64", "f64"),
        ("bool", "bool"),
        ("usize", "usize"),
    ].into_iter().map(|(suffix, target_type)| {
        let ty_ident = syn::parse_str::<Type>(target_type).unwrap();

        let match_arms = fields.iter().filter_map(|field| {
            let ident = field.ident.as_ref()?;
            let field_ty = &field.ty;

            let field_type_str = field_ty.to_token_stream().to_string();
            let is_dynamic_target = field_type_str == format!("Dynamic < {} >", target_type);

            if is_dynamic_target {
                let field_str = ident.to_string();
                Some(quote! {
                    #field_str => self.#ident.set(value),
                })
            } else {
                None
            }
        });

        let trait_ident = quote::format_ident!("SetDynamicField{}", suffix);

        quote! {
            pub trait #trait_ident {
                fn set_dynamic(&mut self, field: &str, value: #ty_ident);
            }

            impl #trait_ident for #name {
                fn set_dynamic(&mut self, field: &str, value: #ty_ident) {
                    match field {
                        #(#match_arms)*
                        _ => panic!("Unknown field: {}", field),
                    }
                }
            }
        }
    });

    let expanded = quote! {
        #(#impls)*
    };

    TokenStream::from(expanded)
}
