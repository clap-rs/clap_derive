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

// creates the clap::Parse impl
pub fn impl_parse(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let parse_impl = gen_parse_impl(&input.ident);
    quote! { #parse_impl }
}

pub fn gen_parse_impl(struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        impl ::clap_derive::clap::Parse for #struct_name { }
    }
}
