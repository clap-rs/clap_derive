use syn;
use quote;

use helpers::{Parser, Ty, AttrSource, ty, sub_type, extract_attrs, from_attr_or_env};
use errors::Result;

fn impl_clap_app_for_struct(name: &syn::Ident, fields: &[syn::Field], attrs: &[syn::Attribute]) -> quote::Tokens {
    let clap = gen_clap_struct(attrs);
    let augment_clap = gen_augment_clap(fields);
    let from_clap = gen_from_clap(name, fields);

    quote! {
        impl _structopt::ClapApp for #name {
            #clap
            #from_clap
        }

        impl #name {
            #augment_clap
        }
    }
}

fn impl_clap_app_for_enum(name: &syn::Ident, variants: &[syn::Variant], attrs: &[syn::Attribute]) -> quote::Tokens {
    if variants.iter().any(|variant| {
            if let syn::VariantData::Tuple(..) = variant.data { true } else { false }
        })
    {
        panic!("enum variants cannot be tuples")
    }

    let clap = gen_clap_enum(attrs);
    let augment_clap = gen_augment_clap_enum(variants);
    let from_clap = gen_from_clap_enum(name);
    let from_subcommand = gen_from_subcommand(name, variants);

    quote! {
        impl _structopt::ClapApp for #name {
            #clap
            #from_clap
        }

        impl #name {
            #augment_clap
            #from_subcommand
        }
    }
}

pub(crate) fn impl_clap_app(ast: &syn::DeriveInput) -> Result<quote::Tokens> {
    let struct_name = &ast.ident;
    let inner_impl = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) =>
            impl_clap_app_for_struct(struct_name, fields, &ast.attrs),
        syn::Body::Enum(ref variants) =>
            impl_clap_app_for_enum(struct_name, variants, &ast.attrs),
        _ => panic!("clap_derive only supports non-tuple structs and enums")
    };

    let dummy_const = syn::Ident::new(format!("_IMPL_CLAP_APP_FOR_{}", struct_name));
    Ok(quote! {
        #[allow(non_upper_case_globals)]
        #[allow(unused_attributes, unused_imports, unused_variables)]
        const #dummy_const: () = {
            extern crate structopt as _structopt;
            use structopt::ClapApp;
            #inner_impl
        };
    })
}

fn is_subcommand(field: &syn::Field) -> bool {
    field.attrs.iter()
        .map(|attr| &attr.value)
        .any(|meta| if let syn::MetaItem::List(ref i, ref l) = *meta {
            if i != "clap" { return false; }
            match l.first() {
                Some(&syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref inner))) => inner == "subcommand",
                _ => false
            }
        } else {
          false
        })
}

fn get_default_parser() -> (Parser, quote::Tokens) {
    (Parser::TryFromStr, quote!(::std::str::FromStr::from_str))
}

fn get_parser(field: &syn::Field) -> Option<(Parser, quote::Tokens)> {
    field.attrs.iter()
        .flat_map(|attr| {
            if let syn::MetaItem::List(ref i, ref l) = attr.value {
                if i == "clap" {
                    return &**l;
                }
            }
            &[]
        })
        .filter_map(|attr| {
            if let syn::NestedMetaItem::MetaItem(syn::MetaItem::List(ref i, ref l)) = *attr {
                if i == "parse" {
                    return l.first();
                }
            }
            None
        })
        .map(|attr| {
            match *attr {
                syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref i, syn::Lit::Str(ref v, _))) => {
                    let function = syn::parse_path(v).expect("parser function path");
                    let parser = if i == "from_str" {
                        Parser::FromStr
                    } else if i == "try_from_str" {
                        Parser::TryFromStr
                    } else if i == "from_os_str" {
                        Parser::FromOsStr
                    } else if i == "try_from_os_str" {
                        Parser::TryFromOsStr
                    } else {
                        panic!("unsupported parser {}", i)
                    };
                    (parser, quote!(#function))
                }
                syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref i)) => {
                    if i == "from_str" {
                        (Parser::FromStr, quote!(::std::convert::From::from))
                    } else if i == "try_from_str" {
                        (Parser::TryFromStr, quote!(::std::str::FromStr::from_str))
                    } else if i == "from_os_str" {
                        (Parser::FromOsStr, quote!(::std::convert::From::from))
                    } else if i == "try_from_os_str" {
                        panic!("cannot omit parser function name with `try_from_os_str`")
                    } else {
                        panic!("unsupported parser {}", i)
                    }
                }
                _ => panic!("unknown value parser specification"),
            }
        })
        .next()
}

fn convert_with_custom_parse(cur_type: Ty) -> Ty {
    match cur_type {
        Ty::Bool | Ty::U64 => Ty::Other,
        rest => rest,
    }
}

/// Generate a block of code to add arguments/subcommands corresponding to
/// the `fields` to an app.
fn gen_augmentation(fields: &[syn::Field], app_var: &syn::Ident) -> quote::Tokens {
    let subcmds: Vec<quote::Tokens> = fields.iter()
        .filter(|&field| is_subcommand(field))
        .map(|field| {
            let cur_type = ty(&field.ty);
            let subcmd_type = match (cur_type, sub_type(&field.ty)) {
                (Ty::Option, Some(sub_type)) => sub_type,
                _ => &field.ty
            };
            let required = if cur_type == Ty::Option {
                quote!()
            } else {
                quote!( let #app_var = #app_var.setting(_structopt::clap::AppSettings::SubcommandRequiredElseHelp); )
            };

            quote!{
                let #app_var = #subcmd_type ::augment_clap( #app_var );
                #required
            }
        })
        .collect();

    assert!(subcmds.len() <= 1, "cannot have more than one nested subcommand");

    let args = fields.iter()
        .filter(|&field| !is_subcommand(field))
        .map(|field| {
            let name = gen_name(field);
            let mut cur_type = ty(&field.ty);
            let convert_type = match cur_type {
                Ty::Vec | Ty::Option => sub_type(&field.ty).unwrap_or(&field.ty),
                _ => &field.ty,
            };

            let parser = get_parser(field);
            if parser.is_some() {
                cur_type = convert_with_custom_parse(cur_type);
            }
            let validator = match parser.unwrap_or_else(get_default_parser) {
                (Parser::TryFromStr, f) => quote! {
                    .validator(|s| {
                        #f(&s)
                            .map(|_: #convert_type| ())
                            .map_err(|e| e.to_string())
                    })
                },
                (Parser::TryFromOsStr, f) => quote! {
                    .validator_os(|s| #f(&s).map(|_: #convert_type| ()))
                },
                _ => quote! {},
            };

            let modifier = match cur_type {
                Ty::Bool => quote!( .takes_value(false).multiple(false) ),
                Ty::U64 => quote!( .takes_value(false).multiple(true) ),
                Ty::Option => quote!( .takes_value(true).multiple(false) #validator ),
                Ty::Vec => quote!( .takes_value(true).multiple(true) #validator ),
                Ty::Other => {
                    let required = extract_attrs(&field.attrs, AttrSource::Field)
                        .find(|&(ref i, _)| i.as_ref() == "default_value"
                              || i.as_ref() == "default_value_raw")
                        .is_none();
                    quote!( .takes_value(true).multiple(false).required(#required) #validator )
                },
            };
            let from_attr = extract_attrs(&field.attrs, AttrSource::Field)
                .filter(|&(ref i, _)| i.as_ref() != "name")
                .map(|(i, l)| gen_attr_call(&i, &l));
            quote!( .arg(_structopt::clap::Arg::with_name(stringify!(#name)) #modifier #(#from_attr)*) )
        });

    quote! {{
        use std::error::Error;
        let #app_var = #app_var #( #args )* ;
        #( #subcmds )*
        #app_var
    }}
}

/// Interpret the value of `*_raw` attributes as code and the rest as strings.
fn gen_attr_call(key: &syn::Ident, val: &syn::Lit) -> quote::Tokens {
    if let syn::Lit::Str(ref val, _) = *val {
        let key = key.as_ref();
        if key.ends_with("_raw") {
            let key = syn::Ident::from(&key[..(key.len() - 4)]);
            // Call method without quoting the string
            let ts = syn::parse_token_trees(val)
                .expect(&format!("bad parameter {} = {}: the parameter must be valid rust code", key, val));
            return quote!(.#key(#(#ts)*));
        }
    }
    quote!(.#key(#val))
}

fn gen_constructor(fields: &[syn::Field]) -> quote::Tokens {
    let fields = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let name = gen_name(field);
        if is_subcommand(field) {
            let cur_type = ty(&field.ty);
            let subcmd_type = match (cur_type, sub_type(&field.ty)) {
                (Ty::Option, Some(sub_type)) => sub_type,
                _ => &field.ty
            };
            let unwrapper = match cur_type {
                Ty::Option => quote!(),
                _ => quote!( .unwrap() )
            };
            quote!( #field_name: #subcmd_type ::from_subcommand(matches.subcommand()) #unwrapper )
        } else {
            let mut cur_type = ty(&field.ty);
            let parser = get_parser(field);
            if parser.is_some() {
                cur_type = convert_with_custom_parse(cur_type);
            }

            let (value_of, values_of, parse) = match parser.unwrap_or_else(get_default_parser) {
                (Parser::FromStr, f) => (
                    quote!(value_of),
                    quote!(values_of),
                    f,
                ),
                (Parser::TryFromStr, f) => (
                    quote!(value_of),
                    quote!(values_of),
                    quote!(|s| #f(s).unwrap()),
                ),
                (Parser::FromOsStr, f) => (
                    quote!(value_of_os),
                    quote!(values_of_os),
                    f,
                ),
                (Parser::TryFromOsStr, f) => (
                    quote!(value_of_os),
                    quote!(values_of_os),
                    quote!(|s| #f(s).unwrap()),
                ),
            };

            let convert = match cur_type {
                Ty::Bool => quote!(is_present(stringify!(#name))),
                Ty::U64 => quote!(occurrences_of(stringify!(#name))),
                Ty::Option => quote! {
                    #value_of(stringify!(#name))
                        .as_ref()
                        .map(#parse)
                },
                Ty::Vec => quote! {
                    #values_of(stringify!(#name))
                        .map(|v| v.map(#parse).collect())
                        .unwrap_or_else(Vec::new)
                },
                Ty::Other => quote! {
                    #value_of(stringify!(#name))
                        .map(#parse)
                        .unwrap()
                },
            };
            quote!( #field_name: matches.#convert )
        }
    });

    quote! {{
        #( #fields ),*
    }}
}

fn gen_name(field: &syn::Field) -> syn::Ident {
    extract_attrs(&field.attrs, AttrSource::Field)
        .filter(|&(ref i, _)| i.as_ref() == "name")
        .last()
        .and_then(|(_, ref l)| match l {
            &syn::Lit::Str(ref s, _) => Some(syn::Ident::new(s.clone())),
            _ => None,
        })
        .unwrap_or_else(|| field.ident.as_ref().unwrap().clone())
}

fn gen_from_clap(struct_name: &syn::Ident, fields: &[syn::Field]) -> quote::Tokens {
    let field_block = gen_constructor(fields);

    quote! {
        fn from_clap(matches: _structopt::clap::ArgMatches) -> Self {
            #struct_name #field_block
        }
    }
}

fn format_author(raw_authors: syn::Lit) -> syn::Lit {
    let raw_authors = match raw_authors {
        syn::Lit::Str(x, _) => x,
        x => return x,
    };
    let authors = raw_authors.replace(":", ", ");
    syn::Lit::Str(authors, syn::StrStyle::Cooked)
}

fn gen_clap(attrs: &[syn::Attribute]) -> quote::Tokens {
    let attrs: Vec<_> = extract_attrs(attrs, AttrSource::Struct).collect();
    let name = from_attr_or_env(&attrs, "name", "CARGO_PKG_NAME");
    let version = from_attr_or_env(&attrs, "version", "CARGO_PKG_VERSION");
    let author = format_author(from_attr_or_env(&attrs, "author", "CARGO_PKG_AUTHORS"));
    let about = from_attr_or_env(&attrs, "about", "CARGO_PKG_DESCRIPTION");
    let settings = attrs.iter()
        .filter(|&&(ref i, _)| !["name", "version", "author", "about"].contains(&i.as_ref()))
        .map(|&(ref i, ref l)| gen_attr_call(i, l))
        .collect::<Vec<_>>();

    quote! {
        _structopt::clap::App::new(#name)
            .version(#version)
            .author(#author)
            .about(#about)
            #( #settings )*
    }
}

fn gen_clap_struct(struct_attrs: &[syn::Attribute]) -> quote::Tokens {
    let gen = gen_clap(struct_attrs);

    quote! {
        fn clap<'a, 'b>() -> _structopt::clap::App<'a, 'b> {
            let app = #gen;
            Self::augment_clap(app)
        }
    }
}

fn gen_augment_clap(fields: &[syn::Field]) -> quote::Tokens {
    let app_var = syn::Ident::new("app");
    let augmentation = gen_augmentation(fields, &app_var);
    quote! {
        pub fn augment_clap<'a, 'b>(#app_var: _structopt::clap::App<'a, 'b>) -> _structopt::clap::App<'a, 'b> {
            #augmentation
        }
    }
}

fn gen_clap_enum(enum_attrs: &[syn::Attribute]) -> quote::Tokens {
    let gen = gen_clap(enum_attrs);
    quote! {
        fn clap<'a, 'b>() -> _structopt::clap::App<'a, 'b> {
            let app = #gen
                .setting(_structopt::clap::AppSettings::SubcommandRequiredElseHelp);
            Self::augment_clap(app)
        }
    }
}

fn gen_augment_clap_enum(variants: &[syn::Variant]) -> quote::Tokens {
    let subcommands = variants.iter().map(|variant| {
        let name = extract_attrs(&variant.attrs, AttrSource::Struct)
            .filter_map(|attr| match attr {
                (ref i, syn::Lit::Str(ref s, ..)) if i == "name" =>
                    Some(s.to_string()),
                _ => None
            })
            .next()
            .unwrap_or_else(|| variant.ident.to_string());
        let app_var = syn::Ident::new("subcommand");
        let arg_block = match variant.data {
            syn::VariantData::Struct(ref fields) => gen_augmentation(fields, &app_var),
            syn::VariantData::Unit => quote!( #app_var ),
            _ => unreachable!()
        };
        let from_attr = extract_attrs(&variant.attrs, AttrSource::Struct)
            .filter(|&(ref i, _)| i != "name")
            .map(|(i, l)| gen_attr_call(&i, &l));

        quote! {
            .subcommand({
                let #app_var = _structopt::clap::SubCommand::with_name( #name )
                    #( #from_attr )* ;
                #arg_block
            })
        }
    });

    quote! {
        pub fn augment_clap<'a, 'b>(app: _structopt::clap::App<'a, 'b>) -> _structopt::clap::App<'a, 'b> {
            app #( #subcommands )*
        }
    }
}

fn gen_from_clap_enum(name: &syn::Ident) -> quote::Tokens {
    quote! {
        #[doc(hidden)]
        fn from_clap(matches: _structopt::clap::ArgMatches) -> Self {
            #name ::from_subcommand(matches.subcommand())
                .unwrap()
        }
    }
}

fn gen_from_subcommand(name: &syn::Ident, variants: &[syn::Variant]) -> quote::Tokens {
    let match_arms = variants.iter().map(|variant| {
        let sub_name = extract_attrs(&variant.attrs, AttrSource::Struct)
            .filter_map(|attr| match attr {
                (ref i, syn::Lit::Str(ref s, ..)) if i == "name" =>
                    Some(s.to_string()),
                _ => None
            })
            .next()
            .unwrap_or_else(|| variant.ident.as_ref().to_string());
        let variant_name = &variant.ident;
        let constructor_block = match variant.data {
            syn::VariantData::Struct(ref fields) => gen_constructor(fields),
            syn::VariantData::Unit => quote!(),  // empty
            _ => unreachable!()
        };

        quote! {
            (#sub_name, Some(matches)) =>
                Some(#name :: #variant_name #constructor_block)
        }
    });

    quote! {
        #[doc(hidden)]
        pub fn from_subcommand<'a, 'b>(sub: (&'b str, Option<&'b _structopt::clap::ArgMatches<'a>>)) -> Option<Self> {
            match sub {
                #( #match_arms ),*,
                _ => None
            }
        }
    }
}