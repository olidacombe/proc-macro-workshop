use proc_macro;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields};

fn required_option_getter(name: &syn::Ident) -> TokenStream {
    let error_msg = format!("Required parameter `{}` not specified", name);
    quote! {
        #name.take().ok_or(#error_msg)?
    }
}

type SubMeta = syn::punctuated::Punctuated<syn::NestedMeta, syn::token::Comma>;

fn get_builder_attr(attrs: &Vec<syn::Attribute>) -> Option<SubMeta> {
    attrs.iter().find_map(|attr| {
        if let Ok(syn::Meta::List(list)) = attr.parse_meta() {
            if list.path.is_ident("builder") {
                return Some(list.nested);
            }
        }
        None
    })
}

fn each_method(sub_meta: &SubMeta) -> Option<syn::Ident> {
    sub_meta
        .iter()
        .filter_map(|s| match s {
            syn::NestedMeta::Meta(meta) => Some(meta),
            _ => None,
        })
        .filter_map(|m| match m {
            syn::Meta::NameValue(nv) => Some(nv),
            _ => None,
        })
        .find_map(|nv| match nv.path.is_ident("each") {
            true => match &nv.lit {
                syn::Lit::Str(s) => {
                    Some(syn::Ident::new(&s.value(), proc_macro2::Span::call_site()))
                }
                _ => None,
            },
            false => None,
        })
}

struct BuilderField {
    ident: syn::Ident,
    ty: syn::Type,
    outer_ty: syn::Type,
    default: proc_macro2::TokenStream,
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
        let mut inner_ty: Option<syn::Type> = None;
        let mut outer_ty: parse_quote! {Option};
        let mut single_ty: Option<syn::Type> = None;
        let mut default = parse_quote! {None};
        if let syn::Type::Path(ref p) = ty {
            if let Some(ref seg) = p.path.segments.first() {
                if let syn::PathArguments::AngleBracketed(ref args) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        if seg.ident == "Option" {
                            field_getter = quote! { #ident.take() };
                            ty = inner.clone();
                        } else {
                            inner_ty = Some(inner.clone());
                        }
                    }
                }
            }
        }
        let mut setters = HashMap::<syn::Ident, proc_macro2::TokenStream>::new();
        setters.insert(
            ident.clone(),
            quote! {
                pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }

            },
        );

        if let Some(builder_attr) = get_builder_attr(&attrs) {
            if let Some(each) = each_method(&builder_attr) {
                if let Some(item_ty) = inner_ty {
                    outer_ty = quote! {Vec};
                    default = quote! {Vec::<#item_ty>::new()};
                    let setter = quote! {
                        pub fn #each(&mut self, item: #item_ty) -> &mut Self {
                            self.#ident.push(item);
                            self
                        }
                    };
                    setters.insert(each, setter);
                    ty = item_ty;
                }
            }
        }

        BuilderField {
            field_getter,
            ident,
            ty,
            outer_ty,
            default,
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
