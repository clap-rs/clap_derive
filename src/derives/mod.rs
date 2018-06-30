// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate is custom derive for StructOpt. It should not be used
//! directly. See [structopt documentation](https://docs.rs/structopt)
//! for the usage of `#[derive(StructOpt)]`.
mod attrs;

use self::attrs::{Attrs, Kind, Parser, Ty};
use proc_macro2::{Span, TokenStream};

use std::env;
use syn;
use syn::punctuated::Punctuated;
use syn::token::Comma;

fn sub_type(t: &syn::Type) -> Option<&syn::Type> {
    let segs = match *t {
        syn::Type::Path(syn::TypePath {
            path: syn::Path { ref segments, .. },
            ..
        }) => segments,
        _ => return None,
    };
    match *segs.iter().last().unwrap() {
        syn::PathSegment {
            arguments:
                syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    ref args, ..
                }),
            ..
        } if args.len() == 1 =>
        {
            if let syn::GenericArgument::Type(ref ty) = args[0] {
                Some(ty)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Generate a block of code to add arguments/subcommands corresponding to
/// the `fields` to an app.
fn gen_augmentation(fields: &Punctuated<syn::Field, Comma>, app_var: &syn::Ident) -> TokenStream {
    let subcmds: Vec<_> = fields
        .iter()
        .filter_map(|field| {
            let attrs = Attrs::from_field(&field);
            if let Kind::Subcommand(ty) = attrs.kind() {
                let subcmd_type = match (ty, sub_type(&field.ty)) {
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
                    Ty::Vec | Ty::Option => sub_type(&field.ty).unwrap_or(&field.ty),
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

fn gen_constructor(fields: &Punctuated<syn::Field, Comma>) -> TokenStream {
    let fields = fields.iter().map(|field| {
        let attrs = Attrs::from_field(field);
        let field_name = field.ident.as_ref().unwrap();
        match attrs.kind() {
            Kind::Subcommand(ty) => {
                let subcmd_type = match (ty, sub_type(&field.ty)) {
                    (Ty::Option, Some(sub_type)) => sub_type,
                    _ => &field.ty,
                };
                let unwrapper = match ty {
                    Ty::Option => quote!(),
                    _ => quote!( .unwrap() ),
                };
                quote!(#field_name: <#subcmd_type>::from_subcommand(matches.subcommand())#unwrapper)
            }
            Kind::FlattenStruct => {
                quote!(#field_name: ::clap_derive::clap::Clap::from_argmatches(matches))
            }
            Kind::Arg(ty) => {
                use self::Parser::*;
                let (value_of, values_of, parse) = match *attrs.parser() {
                    (FromStr, ref f) => (quote!(value_of), quote!(values_of), f.clone()),
                    (TryFromStr, ref f) => (
                        quote!(value_of),
                        quote!(values_of),
                        quote!(|s| #f(s).unwrap()),
                    ),
                    (FromOsStr, ref f) => (quote!(value_of_os), quote!(values_of_os), f.clone()),
                    (TryFromOsStr, ref f) => (
                        quote!(value_of_os),
                        quote!(values_of_os),
                        quote!(|s| #f(s).unwrap()),
                    ),
                    (FromOccurrences, ref f) => (quote!(occurrences_of), quote!(), f.clone()),
                };

                let occurences = attrs.parser().0 == FromOccurrences;
                let name = attrs.name();
                let field_value = match ty {
                    Ty::Bool => quote!(matches.is_present(#name)),
                    Ty::Option => quote! {
                        matches.#value_of(#name)
                            .as_ref()
                            .map(#parse)
                    },
                    Ty::Vec => quote! {
                        matches.#values_of(#name)
                            .map(|v| v.map(#parse).collect())
                            .unwrap_or_else(Vec::new)
                    },
                    Ty::Other if occurences => quote! {
                        #parse(matches.#value_of(#name))
                    },
                    Ty::Other => quote! {
                        matches.#value_of(#name)
                            .map(#parse)
                            .unwrap()
                    },
                };

                quote!( #field_name: #field_value )
            }
        }
    });

    quote! {{
        #( #fields ),*
    }}
}

fn gen_from_clap(struct_name: &syn::Ident, fields: &Punctuated<syn::Field, Comma>) -> TokenStream {
    let field_block = gen_constructor(fields);

    quote! {
        fn from_clap(matches: &::clap_derive::clap::ArgMatches) -> Self {
            #struct_name #field_block
        }
    }
}

fn gen_clap(attrs: &[syn::Attribute]) -> TokenStream {
    let name = env::var("CARGO_PKG_NAME")
        .ok()
        .unwrap_or_else(String::default);
    let attrs = Attrs::from_struct(attrs, name);
    let name = attrs.name();
    let methods = attrs.methods();
    quote!(::clap_derive::clap::App::new(#name)#methods)
}

fn gen_clap_struct(struct_attrs: &[syn::Attribute]) -> TokenStream {
    let gen = gen_clap(struct_attrs);
    quote! {
        fn clap<'a, 'b>() -> ::clap_derive::clap::App<'a, 'b> {
            let app = #gen;
            Self::augment_clap(app)
        }
    }
}

fn gen_augment_clap(fields: &Punctuated<syn::Field, Comma>) -> TokenStream {
    let app_var = syn::Ident::new("app", Span::call_site());
    let augmentation = gen_augmentation(fields, &app_var);
    quote! {
        pub fn augment_clap<'a, 'b>(
            #app_var: ::clap_derive::clap::App<'a, 'b>
        ) -> ::clap_derive::clap::App<'a, 'b> {
            #augmentation
        }
    }
}

fn gen_clap_enum(enum_attrs: &[syn::Attribute]) -> TokenStream {
    let gen = gen_clap(enum_attrs);
    quote! {
        fn clap<'a, 'b>() -> ::clap_derive::clap::App<'a, 'b> {
            let app = #gen
                .setting(::clap_derive::clap::AppSettings::SubcommandRequiredElseHelp);
            Self::augment_clap(app)
        }
    }
}

fn gen_augment_clap_enum(variants: &Punctuated<syn::Variant, Comma>) -> TokenStream {
    use syn::Fields::*;

    let subcommands = variants.iter().map(|variant| {
        let name = variant.ident.to_string();
        let attrs = Attrs::from_struct(&variant.attrs, name);
        let app_var = syn::Ident::new("subcommand", Span::call_site());
        let arg_block = match variant.fields {
            Named(ref fields) => gen_augmentation(&fields.named, &app_var),
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

fn gen_from_clap_enum(name: &syn::Ident) -> TokenStream {
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
) -> TokenStream {
    use syn::Fields::*;

    let match_arms = variants.iter().map(|variant| {
        let attrs = Attrs::from_struct(&variant.attrs, variant.ident.to_string());
        let sub_name = attrs.name();
        let variant_name = &variant.ident;
        let constructor_block = match variant.fields {
            Named(ref fields) => gen_constructor(&fields.named),
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

fn impl_clap_for_struct(
    name: &syn::Ident,
    fields: &Punctuated<syn::Field, Comma>,
    attrs: &[syn::Attribute],
) -> TokenStream {
    let clap = gen_clap_struct(attrs);
    let augment_clap = gen_augment_clap(fields);
    let from_clap = gen_from_clap(name, fields);

    quote! {
        #[allow(unused_variables)]
        impl ::clap_derive::clap::Clap for #name {
            #clap
            #from_clap
        }

        #[allow(dead_code, unreachable_code)]
        #[doc(hidden)]
        impl #name {
            #augment_clap
            pub fn is_subcommand() -> bool { false }
        }
    }
}

fn impl_clap_for_enum(
    name: &syn::Ident,
    variants: &Punctuated<syn::Variant, Comma>,
    attrs: &[syn::Attribute],
) -> TokenStream {
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

pub fn impl_clap(input: &syn::DeriveInput) -> TokenStream {
    use syn::Data::*;

    let struct_name = &input.ident;
    let inner_impl = match input.data {
        Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => impl_clap_for_struct(struct_name, &fields.named, &input.attrs),
        Enum(ref e) => impl_clap_for_enum(struct_name, &e.variants, &input.attrs),
        _ => panic!("structopt only supports non-tuple structs and enums"),
    };

    quote!(#inner_impl)
}
