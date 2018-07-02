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
use std::env;
use syn;

use derives::attrs::Attrs;

// Generates the clap::IntoApp impl
pub fn impl_into_app(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = &input.ident;
    let into_app_fn = gen_into_app(&input.attrs);

    quote! {
        #[allow(unused_variables)]
        impl ::clap_derive::clap::IntoApp for #struct_name {
            #into_app_fn
            // #clap
            // #from_clap
        }

        impl From<#struct_name> for ::clap_derive::clap::App {
            fn from(&self) -> ::clap_derive::clap::App {
                <self as ::clap_derive::clap::IntoApp>::into_app()
            }
        }
    }
}

pub(crate) fn gen_into_app(struct_attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    let built_app = gen_app(struct_attrs);
    quote! {
        fn into_app<'a, 'b>() -> ::clap_derive::clap::App<'a, 'b> {
            let app = #built_app;
            Self::augment_clap(app)
        }
    }
}

pub(crate) fn gen_app(attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    let name = env::var("CARGO_PKG_NAME")
        .ok()
        .unwrap_or_else(String::default);
    let attrs = Attrs::from_struct(attrs, name);
    let name = attrs.name();
    let methods = attrs.methods();
    quote!(::clap_derive::clap::App::new(#name)#methods)
}
