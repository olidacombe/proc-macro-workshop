use proc_macro;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let builder_name = format_ident!("{}Builder", name);

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => fields.named.into_pairs().map(|mut pair| {
                let mut field = pair.value_mut();
                let ty = &field.ty;

                field.ty = parse_quote! {
                    Option<#ty>
                };

                pair
            }),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let expanded = quote! {
        pub struct #builder_name {
            #(#fields)*
        }
        impl #name {
            pub fn builder() {}
        }
    };

    proc_macro::TokenStream::from(expanded)
}
