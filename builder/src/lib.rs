use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let builder_name = format_ident!("{}Builder", name);

    let fields = match input.data {
        Data::Struct(ref s) => match s.fields {
            Fields::Named(ref fields) => fields.named.iter(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let expanded = quote! {
        pub struct #builder_name {
            #(#fields),*
        }
        impl #name {
            pub fn builder() {}
        }
    };

    TokenStream::from(expanded)
}
