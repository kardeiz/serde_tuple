#![recursion_limit = "4096"]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse, parse_macro_input, parse_quote, Attribute, Error, Fields, Index, ItemStruct,
    LifetimeDef, Meta, NestedMeta, Path, Result,
};

struct WrappedItemStruct(ItemStruct);

impl parse::Parse for WrappedItemStruct {
    fn parse(input: parse::ParseStream) -> Result<Self> {
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

#[proc_macro_derive(Serialize_tuple, attributes(serde))]
pub fn derive_serialize_tuple(input: TokenStream) -> TokenStream {
    let WrappedItemStruct(item) = parse_macro_input!(input as WrappedItemStruct);

    let ident = &item.ident;
    // Only keep serde attributes for the inner struct
    let serde_attrs: Vec<_> = item
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("serde"))
        .collect();
    let serde_rename_line = parse_attrs(&item);

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let (field_tys, field_calls): (Vec<_>, Vec<_>) = item
        .fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            // Only keep serde attributes for the inner struct field
            let serde_field_attrs: Vec<_> = field
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("serde"))
                .collect();
            (
                quote!(#(#serde_field_attrs)* &'serde_tuple_inner #ty),
                quote!(&self.#ident),
            )
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
                #[derive(serde::Serialize)]
                #serde_rename_line
                #(#serde_attrs)*
                struct Inner #inner_ty_generics (#(#field_tys,)*);

                let inner = Inner(#(#field_calls,)*);
                serde::Serialize::serialize(&inner, serde_tuple::Serializer(serializer))
            }
        }
    };

    out.into()
}

#[proc_macro_derive(Deserialize_tuple, attributes(serde))]
pub fn derive_deserialize_tuple(input: TokenStream) -> TokenStream {
    let WrappedItemStruct(item) = parse_macro_input!(input as WrappedItemStruct);

    let ident = &item.ident;
    // Only keep serde attributes for the inner struct
    let serde_attrs: Vec<_> = item
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("serde"))
        .collect();
    let serde_rename_line = parse_attrs(&item);
    let (_, ty_generics, where_clause) = item.generics.split_for_impl();

    let (field_tys, field_calls): (Vec<_>, Vec<_>) = item
        .fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let idx = Index::from(idx);
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            // Only keep serde attributes for the inner struct field
            let serde_field_attrs: Vec<_> = field
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("serde"))
                .collect();
            (
                quote!(#(#serde_field_attrs)* #ty),
                quote!(#ident: inner.#idx),
            )
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
            fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                #[derive(serde::Deserialize)]
                #serde_rename_line
                #(#serde_attrs)*
                struct Inner #ty_generics (#(#field_tys,)*);
                let inner: Inner #ty_generics =
                    serde::Deserialize::deserialize(serde_tuple::Deserializer(deserializer))?;
                core::result::Result::Ok(#ident {
                    #(#field_calls,)*
                })
            }
        }
    };

    out.into()
}

fn parse_attrs(item: &ItemStruct) -> Option<proc_macro2::TokenStream> {
    let serde_path: Path = parse_quote!(serde);
    let rename_path: Path = parse_quote!(rename);

    if item
        .attrs
        .iter()
        .flat_map(Attribute::parse_meta)
        .filter_map(|x| match x {
            Meta::List(y) => Some(y),
            _ => None,
        })
        .filter(|x| x.path == serde_path)
        .flat_map(|x| x.nested.into_iter())
        .filter_map(|x| match x {
            NestedMeta::Meta(y) => Some(y),
            NestedMeta::Lit(_) => None,
        })
        .filter_map(|x| match x {
            Meta::NameValue(y) => Some(y),
            _ => None,
        })
        .any(|x| x.path == rename_path)
    {
        None
    } else {
        let ident_str = &item.ident.to_string();
        Some(quote!(#[serde(rename = #ident_str)]))
    }
}
