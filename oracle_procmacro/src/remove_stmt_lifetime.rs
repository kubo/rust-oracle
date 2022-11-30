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
use proc_macro::TokenStream;
use quote::quote;
use syn::visit_mut::{self, VisitMut};
use syn::{parse_str, File, ImplItemMethod, ItemImpl, ItemStruct, Lifetime, LifetimeDef, Type};

struct Visitor {
    stmt_type_with_lifetime: Type,
    stmt_type: Type,
    conn_lifetime: Lifetime,
    is_impl_statement: bool,
    found_conn_lifetime: bool,
}

impl Visitor {
    fn new() -> Visitor {
        Visitor {
            stmt_type_with_lifetime: parse_str::<Type>("Statement<'conn>").unwrap(),
            stmt_type: parse_str::<Type>("Statement").unwrap(),
            conn_lifetime: parse_str::<Lifetime>("'conn").unwrap(),
            is_impl_statement: false,
            found_conn_lifetime: false,
        }
    }
}

impl VisitMut for Visitor {
    fn visit_item_struct_mut(&mut self, i: &mut ItemStruct) {
        i.generics.params.clear();
        visit_mut::visit_item_struct_mut(self, i);
    }

    fn visit_item_impl_mut(&mut self, i: &mut ItemImpl) {
        if *i.self_ty == self.stmt_type_with_lifetime {
            self.is_impl_statement = true;
            i.generics.params.clear();
        }
        visit_mut::visit_item_impl_mut(self, i);
        self.is_impl_statement = false;
    }

    fn visit_impl_item_method_mut(&mut self, i: &mut ImplItemMethod) {
        self.found_conn_lifetime = false;
        visit_mut::visit_impl_item_method_mut(self, i);
        if self.is_impl_statement && self.found_conn_lifetime {
            i.sig
                .generics
                .params
                .push(parse_str::<LifetimeDef>("'conn").unwrap().into());
            self.found_conn_lifetime = false;
        }
    }

    fn visit_lifetime_mut(&mut self, i: &mut Lifetime) {
        if *i == self.conn_lifetime {
            self.found_conn_lifetime = true;
        }
        visit_mut::visit_lifetime_mut(self, i);
    }

    fn visit_type_mut(&mut self, i: &mut Type) {
        if *i == self.stmt_type_with_lifetime {
            *i = self.stmt_type.clone();
        }
        visit_mut::visit_type_mut(self, i);
    }
}

pub fn remove_stmt_lifetime(input: TokenStream) -> TokenStream {
    let mut syntax_tree: File = syn::parse(input).unwrap();
    let mut visitor = Visitor::new();
    visitor.visit_file_mut(&mut syntax_tree);
    quote!(#syntax_tree).into()
}
