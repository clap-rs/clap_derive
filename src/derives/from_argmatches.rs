// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>,
// Andrew Hobden (@hoverbear) <andrew@hoverbear.org>, and
// Kevin Knapp (@kbknapp) <kbknapp@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use proc_macro2;
use syn;

pub fn impl_from_argmatches(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    // use syn::Data::*;

    // let struct_name = &input.ident;
    // let inner_impl = match input.data {
    //     // Regular command that translates to a clap::App
    //     Struct(syn::DataStruct {
    //         fields: syn::Fields::Named(ref fields),
    //         ..
    //     }) => impl_clap_for_struct(struct_name, &fields.named, &input.attrs),
    //     // Enums are used for subcommands
    //     Enum(ref e) => impl_clap_for_enum(struct_name, &e.variants, &input.attrs),
    //     _ => panic!("clap_derive only supports non-tuple structs and enums"),
    // };

    // quote!(#inner_impl)
    unimplemented!()
}
