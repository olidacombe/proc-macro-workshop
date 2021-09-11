use proc_macro;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let builder_name = format_ident!("{}Builder", name);

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => fields.named.into_iter().map(|f| (f.ident, f.ty)),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let (field_name, field_type): (Vec<_>, Vec<_>) = fields.unzip();

    let expanded = quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#field_name: None),*
                }
            }
        }
        pub struct #builder_name {
            #(#field_name: Option<#field_type>),*
        }
        impl #builder_name {
        #(fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
            self.#field_name = Some(#field_name);
            self
        })*
        }
    };
    // impl #builder_name {
    // }
    proc_macro::TokenStream::from(expanded)
}
