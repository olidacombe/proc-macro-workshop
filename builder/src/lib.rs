use proc_macro;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

fn required_option_getter(name: &syn::Ident) -> TokenStream {
    let error_msg = format!("Required parameter `{}` not specified", name);
    quote! {
        #name.take().ok_or(#error_msg)?
    }
}

struct BuilderField {
    ident: syn::Ident,
    ty: syn::Type,
    field_getter: proc_macro2::TokenStream,
    setters: HashMap<syn::Ident, proc_macro2::TokenStream>,
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let builder_name = format_ident!("{}Builder", name);

    let fields: Vec<BuilderField> = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => fields
                .named
                .into_iter()
                .map(|f| (f.attrs, f.ident.unwrap(), f.ty)),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
    .map(|(attrs, ident, mut ty)| {
        let mut field_getter = required_option_getter(&ident);
        if let syn::Type::Path(ref p) = ty {
            if let Some(ref seg) = p.path.segments.first() {
                if seg.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(ref args) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            field_getter = quote! { #ident.take() };
                            ty = inner_ty.clone();
                        };
                    }
                }
            }
        }
        let mut setters = HashMap::<syn::Ident, proc_macro2::TokenStream>::new();
        setters.insert(
            ident.clone(),
            quote! {
                pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident= Some(#ident);
                    self
                }

            },
        );

        // TODO here go through attrs looking for
        // builder(each = ...) and inserting on setters
        // a vec appender whenever found
        // HINT: Meta from parse_meta will be a list with
        // nested meta being a NameValue...

        BuilderField {
            field_getter,
            ident,
            ty,
            setters,
        }
    })
    .collect();

    let field_name: Vec<&syn::Ident> = fields.iter().map(|f| &f.ident).collect();
    let field_inner_type: Vec<&syn::Type> = fields.iter().map(|f| &f.ty).collect();
    let field_name_required: Vec<&proc_macro2::TokenStream> =
        fields.iter().map(|f| &f.field_getter).collect();
    let setters: Vec<&proc_macro2::TokenStream> = fields
        .iter()
        .map(|f| f.setters.values())
        .flatten()
        .collect();

    let expanded = quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#field_name: None),*
                }
            }
        }
        pub struct #builder_name {
            #(#field_name: Option<#field_inner_type>),*
        }
        impl #builder_name {
            #(#setters)*
            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#field_name: self.#field_name_required),*
                })
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}
