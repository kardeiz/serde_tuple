use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Parser, Result};
use syn::{parenthesized, Data, DeriveInput, Expr, Field, Fields, Ident, ItemStruct, Generics, Attribute, Meta, Path, Lit, NestedMeta, parse_quote};
use quote::quote;

pub struct WrappedItemStruct(pub ItemStruct);

impl Parse for WrappedItemStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let call_site = Span::call_site();
        if let Ok(item) = ItemStruct::parse(input) {
            if let Fields::Unnamed(_) = item.fields {
                return Err(Error::new(call_site, "struct fields must be named"));
            }
            if item.generics != Generics::default() {
                return Err(Error::new(call_site, "item generics not currently supported"));
            }
            Ok(WrappedItemStruct(item))
        } else {
            Err(Error::new(call_site, "input must be a struct"))
        }
    }
}

fn get_field_position(attrs: &[Attribute]) -> Option<u64> {
    let serde_tuple_path: Path = parse_quote!(serde_tuple);
    let metas = attrs.iter()
        .filter(|x| x.path == serde_tuple_path)
        .flat_map(|x| Attribute::parse_meta(x) )
        .collect::<Vec<_>>();
    if let Some(position) = metas
        .into_iter()
        .filter_map(|x| match x {
            Meta::List(y) => Some(y),
            _ => None
        })
        .flat_map(|x| x.nested.into_iter() )
        .filter_map(|x| match x {
            NestedMeta::Meta(y) => Some(y),
            _ => None
        })
        .filter_map(|x| match x {
            Meta::NameValue(y) => Some(y),
            _ => None
        })
        .find(|x| x.ident == "position")
        .and_then(|x| match x.lit {
            Lit::Int(y) => Some(y),
            _ => None
        })
    {
        Some(position.value())
    } else {
        None
    }
}

pub fn get_sorted_fields(fields: &Fields) -> Vec<&Field> {
    let mut fields = fields.iter().collect::<Vec<_>>();

    let mut has_specified_position = false;
    let mut has_unspecified_position = false;
    let mut seen = std::collections::HashSet::new();

    fields.sort_by_key(|f| {
        let opt_pos = get_field_position(&f.attrs);
        match opt_pos {
            Some(ref pos) => {
                has_specified_position = true;
                if !seen.insert(*pos) {
                    panic!("`position` must be unique for each field");
                }
            },
            None => {
                has_unspecified_position = true;
            }
        }  
        opt_pos
    });

    if has_specified_position && has_unspecified_position {
        panic!("`position` must be used for all fields or none");
    }

    fields
}