#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::*;

mod parse;

use crate::parse::WrappedItemStruct;

#[proc_macro_derive(SerializeTuple, attributes(serde_tuple))]
pub fn derive_serialize_tuple(input: TokenStream) -> TokenStream {
    let WrappedItemStruct(item) = parse_macro_input!(input as WrappedItemStruct);

    let ident = &item.ident;

    let ident_str = &ident.to_string();

    let fields = parse::get_sorted_fields(&item.fields);

    let (field_tys, field_calls): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            (quote!(&'a #ty), quote!(&self.#ident))
        })
        .unzip();

    let out = quote! {
        impl serde::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer
            {
                #[derive(serde_derive::Serialize)]
                #[serde(rename = #ident_str)]
                struct Inner<'a>(#(#field_tys,)*);

                let inner = Inner(#(#field_calls,)*);
                serde::Serialize::serialize(&inner, serializer)
            }
        }
    };

    out.into()
}

#[proc_macro_derive(DeserializeTuple, attributes(serde_tuple))]
pub fn derive_deserialize_tuple(input: TokenStream) -> TokenStream {
    let WrappedItemStruct(item) = parse_macro_input!(input as WrappedItemStruct);

    let ident = &item.ident;

    let ident_str = &ident.to_string();

    let fields = parse::get_sorted_fields(&item.fields);

    let (field_tys, field_calls): (Vec<_>, Vec<_>) = fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            (quote!(#ty), quote!(#ident: inner.#idx))
        })
        .unzip();

    let out = quote! {
        impl<'de> serde::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                #[derive(serde_derive::Deserialize)]
                #[serde(rename = #ident_str)]
                struct Inner(#(#field_tys,)*);

                let inner: Inner = serde::Deserialize::deserialize(deserializer)?;
                Ok(#ident {
                    #(#field_calls,)*
                })
            }
        }
    };

    out.into()
}

#[proc_macro_derive(DeserializeTupleOrNot, attributes(serde, serde_tuple))]
pub fn derive_deserialize_tuple_or_not(input: TokenStream) -> TokenStream {
    let WrappedItemStruct(item) = parse_macro_input!(input as WrappedItemStruct);

    let ident = &item.ident;

    let mut inner = item.clone();
    inner.ident = Ident::new("Inner", Span::call_site());

    for field in inner.fields.iter_mut() {
        let serde_tuple_attr: Path = parse_quote!(serde_tuple);
        field.attrs = field.attrs.drain(..).filter(|x| x.path != serde_tuple_attr).collect();
    }

    let mut inner_as_tup = item.clone();
    inner_as_tup.ident = Ident::new("InnerAsTup", Span::call_site());

    for field in inner_as_tup.fields.iter_mut() {
        let serde_attr: Path = parse_quote!(serde);
        field.attrs = field.attrs.drain(..).filter(|x| x.path != serde_attr).collect();
    }

    let fields = &item.fields.iter().map(|x| x.ident.as_ref().unwrap()).collect::<Vec<_>>();

    let out = quote! {
        impl<'de> serde::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                #[derive(serde_derive::Deserialize)]
                #inner

                #[derive(DeserializeTuple)]
                #inner_as_tup

                #[derive(serde_derive::Deserialize)]
                #[serde(untagged)]
                pub enum EitherInner {
                    Inner(Inner),
                    InnerAsTup(InnerAsTup)
                }

                let either: EitherInner = serde::Deserialize::deserialize(deserializer)?;

                match either {
                    EitherInner::Inner(Inner { #(#fields,)* }) => Ok(#ident { #(#fields,)* }),
                    EitherInner::InnerAsTup(InnerAsTup { #(#fields,)* }) => Ok(#ident { #(#fields,)* }),
                }

            }
        }
    };

    out.into()
}
