// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2022 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------
use darling::ToTokens;
use proc_macro::TokenStream;
use proc_macro2::{Group, Literal, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{
    self, parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, Lit, Meta, MetaList,
    MetaNameValue, NestedMeta, Path,
};

pub fn derive_row_value(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let fields: Vec<_> = if let Data::Struct(DataStruct {
        fields: Fields::Named(named),
        ..
    }) = data
    {
        named
            .named
            .iter()
            .map(|field| {
                let attrs = Attributes::from_field(field);

                let ident = field.ident.as_ref().unwrap();
                let param = Literal::string(
                    &attrs
                        .rename
                        .unwrap_or_else(|| ident.to_string().to_uppercase()),
                );
                let get = if let Some(function_name) = attrs.with {
                    quote! { #function_name(row, #param) }
                } else {
                    quote! { row.get(#param) }
                };

                quote! {
                    #ident: #get?,
                }
            })
            .collect()
    } else {
        panic!("Expected a structure with named fields only");
    };

    let output = quote! {
        impl oracle::RowValue for #ident {
            fn get(row: &oracle::Row) -> oracle::Result<Self> {
                let result = #ident {
                    #(#fields)*
                };
                ::std::result::Result::Ok(result)
            }
        }
    };
    output.into()
}

struct Attributes {
    rename: Option<String>,
    with: Option<Path>,
}

impl Attributes {
    fn from_field(field: &Field) -> Attributes {
        let mut rename: Option<String> = None;
        let mut with: Option<Path> = None;

        for option in field.attrs.iter() {
            match option.parse_meta().unwrap() {
                Meta::List(MetaList { path, nested, .. })
                    if path.to_token_stream().to_string() == "row_value" =>
                {
                    for meta in nested.into_iter() {
                        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            ref path,
                            lit: Lit::Str(ref lit),
                            ..
                        })) = meta
                        {
                            match path.to_token_stream().to_string().as_str() {
                                "rename" => rename = Some(lit.value()),
                                "with" => {
                                    let stream = syn::parse_str(&lit.value());
                                    let path = stream
                                        .and_then(|stream| syn::parse2(respan(stream, lit.span())));

                                    if let Ok(path) = path {
                                        with = Some(path)
                                    }
                                }
                                attr => panic!("Unexpected attribute: '{}'", attr),
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Attributes { rename, with }
    }
}

fn respan(stream: TokenStream2, span: Span) -> TokenStream2 {
    stream
        .into_iter()
        .map(|token| respan_token(token, span))
        .collect()
}

fn respan_token(mut token: TokenTree, span: Span) -> TokenTree {
    if let TokenTree::Group(g) = &mut token {
        *g = Group::new(g.delimiter(), respan(g.stream(), span));
    }
    token.set_span(span);
    token
}
