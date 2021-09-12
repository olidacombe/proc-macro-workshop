use proc_macro;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

// TODO also take type and return empty ident if type
// is optional itself?
fn required_option(name: &syn::Ident) -> TokenStream {
    let error_msg = format!("Required parameter `{}` not specified", name);
    quote! {
        #name.take().ok_or(#error_msg)
    }
}

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
    let field_name_required = field_name
        .iter()
        .map(|ref n| required_option(&n.as_ref().unwrap()));

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
            #(pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                self.#field_name = Some(#field_name);
                self
            })*
            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#field_name: self.#field_name_required?),*
                })
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}
