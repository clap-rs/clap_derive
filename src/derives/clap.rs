// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>,
// Andrew Hobden (@hoverbear) <andrew@hoverbear.org>, and
// Kevin Knapp (@kbknapp) <kbknapp@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// This work was derived from
// [`structopt@master#d983649822`](https://github.com/TeXitoi/structopt/commit/d983649822b32bb6c11fb3ef9891f66258a6e5c9)
// which is licensed under the MIT/Apache 2.0.

//! This crate is custom derive for `clap`. It should not be used
//! directly. See [`clap` custom derive documentation](https://docs.rs/clap)
//! for the usage of `#[derive(Clap)]`.
use derives::attrs::{Attrs, Kind, Parser, Ty};
use proc_macro2;

use syn;
use syn::punctuated::Punctuated;
use syn::token::Comma;

use derives;
use derives::into_app;
use derives::parse;

pub fn impl_clap(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    use syn::Data::*;

    let struct_name = &input.ident;
    let inner_impl = match input.data {
        // Regular command that translates to a clap::App
        Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => impl_clap_for_struct(struct_name, &fields.named, &input.attrs),
        // Enums are used for subcommands
        Enum(ref e) => impl_clap_for_enum(struct_name, &e.variants, &input.attrs),
        _ => panic!("clap_derive only supports non-tuple structs and enums"),
    };

    quote!(#inner_impl)
}

fn impl_clap_for_struct(
    name: &syn::Ident,
    fields: &Punctuated<syn::Field, Comma>,
    attrs: &[syn::Attribute],
) -> proc_macro2::TokenStream {
    let into_app_impl = into_app::gen_into_app(attrs);
    let build_app_fn = gen_build_app(fields);
    let parse_impl = parse::gen_parse(name, fields);

    quote! {
        #[allow(unused_variables)]
        impl ::clap_derive::clap::Clap for #name { }
        //     #clap
        //     #from_clap
        // }

        #into_app_impl

        #parse_impl

        #[allow(dead_code, unreachable_code)]
        #[doc(hidden)]
        impl #name {
            #build_app_fn
            pub fn is_subcommand() -> bool { false }
        }
    }
}

// Generates all the "settings" and utilizes all the builder methods of clap::App
fn gen_build_app(fields: &Punctuated<syn::Field, Comma>) -> proc_macro2::TokenStream {
    let app_var = syn::Ident::new("app", proc_macro2::Span::call_site());
    let builder = gen_builder(fields, &app_var);
    quote! {
        pub fn augment_clap<'a, 'b>(
            #app_var: ::clap_derive::clap::App<'a, 'b>
        ) -> ::clap_derive::clap::App<'a, 'b> {
            #builder
        }
    }
}

/// Generate a block of code to add arguments/subcommands corresponding to
/// the `fields` in custom struct.
fn gen_builder(
    fields: &Punctuated<syn::Field, Comma>,
    app_var: &syn::Ident,
) -> proc_macro2::TokenStream {
    let subcmds: Vec<_> = fields
        .iter()
        .filter_map(|field| {
            let attrs = Attrs::from_field(&field);
            if let Kind::Subcommand(ty) = attrs.kind() {
                let subcmd_type = match (ty, derives::sub_type(&field.ty)) {
                    (Ty::Option, Some(sub_type)) => sub_type,
                    _ => &field.ty,
                };
                let required = if ty == Ty::Option {
                    quote!()
                } else {
                    quote! {
                        let #app_var = #app_var.setting(
                            ::clap_derive::clap::AppSettings::SubcommandRequiredElseHelp
                        );
                    }
                };

                Some(quote!{
                    let #app_var = <#subcmd_type>::augment_clap( #app_var );
                    #required
                })
            } else {
                None
            }
        })
        .collect();

    assert!(
        subcmds.len() <= 1,
        "cannot have more than one nested subcommand"
    );

    let args = fields.iter().filter_map(|field| {
        let attrs = Attrs::from_field(field);
        match attrs.kind() {
            Kind::Subcommand(_) => None,
            Kind::FlattenStruct => {
                let ty = &field.ty;
                Some(quote! {
                    let #app_var = <#ty>::augment_clap(#app_var);
                    let #app_var = if <#ty>::is_subcommand() {
                        #app_var.setting(::clap_derive::clap::AppSettings::SubcommandRequiredElseHelp)
                    } else {
                        #app_var
                    };
                })
            }
            Kind::Arg(ty) => {
                let convert_type = match ty {
                    Ty::Vec | Ty::Option => derives::sub_type(&field.ty).unwrap_or(&field.ty),
                    _ => &field.ty,
                };

                let occurences = attrs.parser().0 == Parser::FromOccurrences;

                let validator = match *attrs.parser() {
                    (Parser::TryFromStr, ref f) => quote! {
                        .validator(|s| {
                            #f(&s)
                            .map(|_: #convert_type| ())
                            .map_err(|e| e.to_string())
                        })
                    },
                    (Parser::TryFromOsStr, ref f) => quote! {
                        .validator_os(|s| #f(&s).map(|_: #convert_type| ()))
                    },
                    _ => quote!(),
                };

                let modifier = match ty {
                    Ty::Bool => quote!( .takes_value(false).multiple(false) ),
                    Ty::Option => quote!( .takes_value(true).multiple(false) #validator ),
                    Ty::Vec => quote!( .takes_value(true).multiple(true) #validator ),
                    Ty::Other if occurences => quote!( .takes_value(false).multiple(true) ),
                    Ty::Other => {
                        let required = !attrs.has_method("default_value");
                        quote!( .takes_value(true).multiple(false).required(#required) #validator )
                    }
                };
                let methods = attrs.methods();
                let name = attrs.name();
                Some(quote!{
                    let #app_var = #app_var.arg(
                        ::clap_derive::clap::Arg::with_name(#name)
                            #modifier
                            #methods
                    );
                })
            }
        }
    });

    quote! {{
        #( #args )*
        #( #subcmds )*
        #app_var
    }}
}

fn gen_clap_enum(enum_attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    let built_app = into_app::gen_app(enum_attrs);
    quote! {
        fn clap<'a, 'b>() -> ::clap_derive::clap::App<'a, 'b> {
            let app = #built_app
                .setting(::clap_derive::clap::AppSettings::SubcommandRequiredElseHelp);
            Self::augment_clap(app)
        }
    }
}

fn gen_augment_clap_enum(variants: &Punctuated<syn::Variant, Comma>) -> proc_macro2::TokenStream {
    use syn::Fields::*;

    let subcommands = variants.iter().map(|variant| {
        let name = variant.ident.to_string();
        let attrs = Attrs::from_struct(&variant.attrs, name);
        let app_var = syn::Ident::new("subcommand", proc_macro2::Span::call_site());
        let arg_block = match variant.fields {
            Named(ref fields) => gen_builder(&fields.named, &app_var),
            Unit => quote!( #app_var ),
            Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) if unnamed.len() == 1 => {
                let ty = &unnamed[0];
                quote! {
                    {
                        let #app_var = <#ty>::augment_clap(#app_var);
                        if <#ty>::is_subcommand() {
                            #app_var.setting(
                                ::clap_derive::clap::AppSettings::SubcommandRequiredElseHelp
                            )
                        } else {
                            #app_var
                        }
                    }
                }
            }
            Unnamed(..) => panic!("{}: tuple enum are not supported", variant.ident),
        };

        let name = attrs.name();
        let from_attrs = attrs.methods();
        quote! {
            .subcommand({
                let #app_var = ::clap_derive::clap::SubCommand::with_name(#name);
                let #app_var = #arg_block;
                #app_var#from_attrs
            })
        }
    });

    quote! {
        pub fn augment_clap<'a, 'b>(
            app: ::clap_derive::clap::App<'a, 'b>
        ) -> ::clap_derive::clap::App<'a, 'b> {
            app #( #subcommands )*
        }
    }
}

fn gen_from_clap_enum(name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        fn from_clap(matches: &::clap_derive::clap::ArgMatches) -> Self {
            <#name>::from_subcommand(matches.subcommand())
                .unwrap()
        }
    }
}

fn gen_from_subcommand(
    name: &syn::Ident,
    variants: &Punctuated<syn::Variant, Comma>,
) -> proc_macro2::TokenStream {
    use syn::Fields::*;

    let match_arms = variants.iter().map(|variant| {
        let attrs = Attrs::from_struct(&variant.attrs, variant.ident.to_string());
        let sub_name = attrs.name();
        let variant_name = &variant.ident;
        let constructor_block = match variant.fields {
            Named(ref fields) => parse::gen_constructor(&fields.named),
            Unit => quote!(),
            Unnamed(ref fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0];
                quote!( ( <#ty as ::clap_derive::clap::Clap>::from_argmatches(matches) ) )
            }
            Unnamed(..) => panic!("{}: tuple enum are not supported", variant.ident),
        };

        quote! {
            (#sub_name, Some(matches)) =>
                Some(#name :: #variant_name #constructor_block)
        }
    });

    quote! {
        pub fn from_subcommand<'a, 'b>(
            sub: (&'b str, Option<&'b ::clap_derive::clap::ArgMatches<'a>>)
        ) -> Option<Self> {
            match sub {
                #( #match_arms ),*,
                _ => None
            }
        }
    }
}

fn impl_clap_for_enum(
    name: &syn::Ident,
    variants: &Punctuated<syn::Variant, Comma>,
    attrs: &[syn::Attribute],
) -> proc_macro2::TokenStream {
    let clap = gen_clap_enum(attrs);
    let augment_clap = gen_augment_clap_enum(variants);
    let from_clap = gen_from_clap_enum(name);
    let from_subcommand = gen_from_subcommand(name, variants);

    quote! {
        impl ::clap_derive::clap::Clap for #name {
            #clap
            #from_clap
        }

        #[allow(unused_variables, dead_code, unreachable_code)]
        #[doc(hidden)]
        impl #name {
            #augment_clap
            #from_subcommand
            pub fn is_subcommand() -> bool { true }
        }
    }
}
