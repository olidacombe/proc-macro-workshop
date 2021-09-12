use proc_macro;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

// TODO also take type and return empty ident if type
// is optional itself?
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
}

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let builder_name = format_ident!("{}Builder", name);

    let fields: Vec<BuilderField> = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => fields.named.into_iter().map(|f| (f.ident.unwrap(), f.ty)),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
    .map(|(ident, ty)| {
        if let syn::Type::Path(ref p) = ty {
            if let Some(ref seg) = p.path.segments.first() {
                if seg.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(ref args) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(ty)) = args.args.first() {
                            return BuilderField {
                                field_getter: quote! { #ident.take() },
                                ident,
                                ty: ty.clone(),
                            };
                        }
                    }
                }
            }
        }
        BuilderField {
            field_getter: required_option_getter(&ident),
            ident,
            ty,
        }
    })
    .collect();

    let field_name: Vec<&syn::Ident> = fields.iter().map(|f| &f.ident).collect();
    let field_inner_type: Vec<&syn::Type> = fields.iter().map(|f| &f.ty).collect();
    let field_name_required: Vec<&proc_macro2::TokenStream> =
        fields.iter().map(|f| &f.field_getter).collect();

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
            #(pub fn #field_name(&mut self, #field_name: #field_inner_type) -> &mut Self {
                self.#field_name = Some(#field_name);
                self
            })*
            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#field_name: self.#field_name_required),*
                })
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}
