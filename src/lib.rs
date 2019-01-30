//! Derive macros to serialize and deserialize struct with named fields as an array of values
//!
//! # Examples
//!
//! ```
//! use serde_tuple::*;
//! 
//! #[derive(Serialize_tuple, Deserialize_tuple)]
//! pub struct Foo<'a> {
//!     bar: &'a str,
//!     baz: i32
//! }

//! let foo = Foo { bar: "Yes", baz: 22 };
//! let json = serde_json::to_string(&foo).unwrap();
//! println!("{}", &json);
//! // # => ["Yes",22]
//! ```

#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::*;

struct WrappedItemStruct(pub ItemStruct);

impl parse::Parse for WrappedItemStruct {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let call_site = Span::call_site();
        if let Ok(item) = ItemStruct::parse(input) {
            if let Fields::Unnamed(_) = item.fields {
                Err(Error::new(call_site, "struct fields must be named"))
            } else {
                Ok(WrappedItemStruct(item))
            }
        } else {
            Err(Error::new(call_site, "input must be a struct"))
        }
    }
}

#[proc_macro_derive(Serialize_tuple, attributes(serde_tuple))]
pub fn derive_serialize_tuple(input: TokenStream) -> TokenStream {
    let WrappedItemStruct(item) = parse_macro_input!(input as WrappedItemStruct);

    let ident = &item.ident;
    let ident_str = &ident.to_string();
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let (field_tys, field_calls): (Vec<_>, Vec<_>) = item
        .fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            (quote!(&'serde_tuple_inner #ty), quote!(&self.#ident))
        })
        .unzip();

    let mut inner_generics = item.generics.clone();

    let inner_lifetime_def: LifetimeDef = parse_quote!('serde_tuple_inner);

    inner_generics.params.push(inner_lifetime_def.into());

    let (_, inner_ty_generics, _) = inner_generics.split_for_impl();

    let out = quote! {
        impl #impl_generics serde::Serialize for #ident #ty_generics #where_clause {
            fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer
            {
                #[derive(serde_derive::Serialize)]
                #[serde(rename = #ident_str)]
                struct Inner #inner_ty_generics (#(#field_tys,)*);

                let inner = Inner(#(#field_calls,)*);
                serde::Serialize::serialize(&inner, serializer)
            }
        }
    };

    out.into()
}

#[proc_macro_derive(Deserialize_tuple, attributes(serde_tuple))]
pub fn derive_deserialize_tuple(input: TokenStream) -> TokenStream {
    let WrappedItemStruct(item) = parse_macro_input!(input as WrappedItemStruct);

    let ident = &item.ident;

    let ident_str = &ident.to_string();
    let (_, ty_generics, where_clause) = item.generics.split_for_impl();

    let (field_tys, field_calls): (Vec<_>, Vec<_>) = item
        .fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            (quote!(#ty), quote!(#ident: inner.#idx))
        })
        .unzip();

    let mut de_generics = item.generics.clone();

    let de_generics_lifetimes = de_generics.lifetimes().collect::<Vec<_>>();

    let de_lifetime_def: LifetimeDef = if de_generics_lifetimes.is_empty() {
        parse_quote!('de)
    } else {
        parse_quote!('de: #(#de_generics_lifetimes)+*)
    };

    de_generics.params.push(de_lifetime_def.into());

    let (de_impl_generics, ..) = de_generics.split_for_impl();

    let out = quote! {
        impl #de_impl_generics serde::Deserialize<'de> for #ident #ty_generics #where_clause {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                #[derive(serde_derive::Deserialize)]
                #[serde(rename = #ident_str)]
                struct Inner #ty_generics (#(#field_tys,)*);
                let inner: Inner #ty_generics = serde::Deserialize::deserialize(deserializer)?;
                Ok(#ident {
                    #(#field_calls,)*
                })
            }
        }
    };

    out.into()
}
