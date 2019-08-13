// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>,
// Kevin Knapp (@kbknapp) <kbknapp@gmail.com>, and
// Andrew Hobden (@hoverbear) <andrew@hoverbear.org>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// This work was derived from Structopt (https://github.com/TeXitoi/structopt)
// commit#ea76fa1b1b273e65e3b0b1046643715b49bec51f which is licensed under the
// MIT/Apache 2.0 license.

use heck::{CamelCase, KebabCase, MixedCase, ShoutySnakeCase, SnakeCase};
use proc_macro2;
use proc_macro_error::span_error;
use std::{env, mem};
use syn;
use syn::spanned::Spanned as _;

use derives;
use derives::spanned::Sp;

/// Default casing style for generated arguments.
pub const DEFAULT_CASING: CasingStyle = CasingStyle::Kebab;

#[derive(Clone, Debug)]
pub enum Kind {
    Arg(Sp<Ty>),
    Subcommand(Sp<Ty>),
    FlattenStruct,
    Skip,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Ty {
    Bool,
    Vec,
    Option,
    OptionOption,
    OptionVec,
    Other,
}

pub struct Attrs {
    name: Sp<String>,
    cased_name: String,
    casing: Sp<CasingStyle>,
    methods: Vec<Method>,
    parser: Sp<(Sp<Parser>, proc_macro2::TokenStream)>,
    has_custom_parser: bool,
    kind: Sp<Kind>,
}

pub struct Method {
    name: syn::Ident,
    args: proc_macro2::TokenStream,
}

#[derive(Debug, PartialEq)]
pub enum Parser {
    FromStr,
    TryFromStr,
    FromOsStr,
    TryFromOsStr,
    FromOccurrences,
}

/// Defines the casing for the attributes long representation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CasingStyle {
    /// Indicate word boundaries with uppercase letter, excluding the first word.
    Camel,
    /// Keep all letters lowercase and indicate word boundaries with hyphens.
    Kebab,
    /// Indicate word boundaries with uppercase letter, including the first word.
    Pascal,
    /// Keep all letters uppercase and indicate word boundaries with underscores.
    ScreamingSnake,
    /// Keep all letters lowercase and indicate word boundaries with underscores.
    Snake,
    /// Use the original attribute name defined in the code.
    Verbatim,
}

/// Output for the gen_xxx() methods were we need more than a simple stream of tokens.
///
/// The output of a generation method is not only the stream of new tokens but also the attribute
/// information of the current element. These attribute information may contain valuable information
/// for any kind of child arguments.
pub struct GenOutput {
    pub tokens: proc_macro2::TokenStream,
    pub attrs: Attrs,
}

impl Parser {
    fn from_ident(ident: syn::Ident) -> Sp<Self> {
        use self::Parser::*;

        let p = |kind| Sp::new(kind, ident.span());
        match &*ident.to_string() {
            "from_str" => p(FromStr),
            "try_from_str" => p(TryFromStr),
            "from_os_str" => p(FromOsStr),
            "try_from_os_str" => p(TryFromOsStr),
            "from_occurrences" => p(FromOccurrences),
            s => span_error!(ident.span(), "unsupported parser `{}`", s),
        }
    }
}

impl CasingStyle {
    fn translate(&self, input: &str) -> String {
        use self::CasingStyle::*;

        match self {
            Pascal => input.to_camel_case(),
            Kebab => input.to_kebab_case(),
            Camel => input.to_mixed_case(),
            ScreamingSnake => input.to_shouty_snake_case(),
            Snake => input.to_snake_case(),
            Verbatim => String::from(input),
        }
    }

    fn from_lit(name: syn::LitStr) -> Sp<Self> {
        use self::CasingStyle::*;

        let normalized = name.value().to_camel_case().to_lowercase();
        let cs = |kind| Sp::new(kind, name.span());

        match normalized.as_ref() {
            "camel" | "camelcase" => cs(Camel),
            "kebab" | "kebabcase" => cs(Kebab),
            "pascal" | "pascalcase" => cs(Pascal),
            "screamingsnake" | "screamingsnakecase" => cs(ScreamingSnake),
            "snake" | "snakecase" => cs(Snake),
            "verbatim" | "verbatimcase" => cs(Verbatim),
            s => span_error!(name.span(), "unsupported casing: `{}`", s),
        }
    }
}

impl Attrs {
    fn new(name: Sp<String>, casing: Sp<CasingStyle>) -> Self {
        let cased_name = casing.translate(&name);

        Self {
            name,
            cased_name,
            casing,
            methods: vec![],
            parser: Sp::call_site((
                Sp::call_site(Parser::TryFromStr),
                quote!(::std::str::FromStr::from_str),
            )),
            has_custom_parser: false,
            kind: Sp::call_site(Kind::Arg(Sp::call_site(Ty::Other))),
        }
    }

    fn push_str_method(&mut self, name: Sp<String>, arg: Sp<String>) {
        match (&**name, &**arg) {
            ("about", "") | ("version", "") | ("author", "") => {
                let methods = mem::replace(&mut self.methods, vec![]);
                self.methods = methods.into_iter().filter(|m| m.name != name).collect();
            }
            ("name", _) => {
                self.cased_name = self.casing.translate(&arg);
                self.name = arg;
            }
            _ => self.methods.push(Method {
                name: name.as_ident(),
                args: quote!(#arg),
            }),
        }
    }

    fn push_attrs(&mut self, attrs: &[syn::Attribute]) {
        use derives::parse::ClapAttr::*;

        for attr in derives::parse::parse_clap_attributes(attrs) {
            match attr {
                Short(ident) => {
                    let cased_name = Sp::call_site(self.cased_name.clone());
                    self.push_str_method(ident.into(), cased_name);
                }

                Long(ident) => {
                    let cased_name = Sp::call_site(self.cased_name.clone());
                    self.push_str_method(ident.into(), cased_name);
                }

                Subcommand(ident) => {
                    let ty = Sp::call_site(Ty::Other);
                    let kind = Sp::new(Kind::Subcommand(ty), ident.span());
                    self.set_kind(kind);
                }

                Flatten(ident) => {
                    let kind = Sp::new(Kind::FlattenStruct, ident.span());
                    self.set_kind(kind);
                }

                Skip(ident) => {
                    let kind = Sp::new(Kind::Skip, ident.span());
                    self.set_kind(kind);
                }

                NameLitStr(name, lit) => {
                    self.push_str_method(name.into(), lit.into());
                }

                NameExpr(name, expr) => self.methods.push(Method {
                    name: name.into(),
                    args: quote!(#expr),
                }),

                MethodCall(name, args) => self.methods.push(Method {
                    name: name.into(),
                    args: quote!(#args),
                }),

                RenameAll(_, casing_lit) => {
                    self.casing = CasingStyle::from_lit(casing_lit);
                    self.cased_name = self.casing.translate(&self.name);
                }

                Parse(ident, spec) => {
                    self.has_custom_parser = true;

                    self.parser = match spec.parse_func {
                        None => {
                            use self::Parser::*;

                            let parser: Sp<_> = Parser::from_ident(spec.kind).into();
                            let function = match *parser {
                                FromStr | FromOsStr => quote!(::std::convert::From::from),
                                TryFromStr => quote!(::std::str::FromStr::from_str),
                                TryFromOsStr => span_error!(
                                    parser.span(),
                                    "cannot omit parser function name with `try_from_os_str`"
                                ),
                                FromOccurrences => quote!({ |v| v as _ }),
                            };
                            Sp::new((parser, function), ident.span())
                        }

                        Some(func) => {
                            let parser: Sp<_> = Parser::from_ident(spec.kind).into();
                            match func {
                                syn::Expr::Path(_) => {
                                    Sp::new((parser, quote!(#func)), ident.span())
                                }
                                _ => span_error!(
                                    func.span(),
                                    "`parse` argument must be a function path"
                                ),
                            }
                        }
                    }
                }
            }
        }
    }

    fn push_doc_comment(&mut self, attrs: &[syn::Attribute], name: &str) {
        let doc_comments = attrs
            .iter()
            .filter_map(|attr| {
                if attr.path.is_ident("doc") {
                    attr.interpret_meta()
                } else {
                    None
                }
            })
            .filter_map(|attr| {
                use syn::Lit::*;
                use syn::Meta::*;
                if let NameValue(syn::MetaNameValue {
                    ident, lit: Str(s), ..
                }) = attr
                {
                    if ident != "doc" {
                        return None;
                    }
                    let value = s.value();
                    let text = value
                        .trim_start_matches("//!")
                        .trim_start_matches("///")
                        .trim_start_matches("/*!")
                        .trim_start_matches("/**")
                        .trim_end_matches("*/")
                        .trim();
                    if text.is_empty() {
                        Some("\n\n".to_string())
                    } else {
                        Some(text.to_string())
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if doc_comments.is_empty() {
            return;
        }
        let merged_lines = doc_comments
            .join(" ")
            .split('\n')
            .map(str::trim)
            .map(str::to_string)
            .collect::<Vec<_>>()
            .join("\n");

        let expected_doc_comment_split = if let Some(content) = doc_comments.get(1) {
            (doc_comments.len() > 2) && (content == &"\n\n")
        } else {
            false
        };

        if expected_doc_comment_split {
            let long_name = Sp::call_site(format!("long_{}", name));

            self.methods.push(Method {
                name: long_name.as_ident(),
                args: quote!(#merged_lines),
            });

            // Remove trailing whitespace and period from short help, as rustdoc
            // best practice is to use complete sentences, but command-line help
            // typically omits the trailing period.
            let short_arg = doc_comments
                .first()
                .map(|s| s.trim())
                .map_or("", |s| s.trim_end_matches('.'));

            self.methods.push(Method {
                name: syn::Ident::new(name, proc_macro2::Span::call_site()),
                args: quote!(#short_arg),
            });
        } else {
            self.methods.push(Method {
                name: syn::Ident::new(name, proc_macro2::Span::call_site()),
                args: quote!(#merged_lines),
            });
        }
    }
    pub fn from_struct(
        attrs: &[syn::Attribute],
        name: Sp<String>,
        argument_casing: Sp<CasingStyle>,
    ) -> Self {
        let mut res = Self::new(name, argument_casing);
        let attrs_with_env = [
            ("version", "CARGO_PKG_VERSION"),
            ("author", "CARGO_PKG_AUTHORS"),
        ];
        attrs_with_env
            .iter()
            .filter_map(|&(m, v)| env::var(v).ok().and_then(|arg| Some((m, arg))))
            .filter(|&(_, ref arg)| !arg.is_empty())
            .for_each(|(name, arg)| {
                let new_arg = if name == "author" {
                    arg.replace(":", ", ")
                } else {
                    arg
                };
                let name = Sp::call_site(name.to_string());
                let new_arg = Sp::call_site(new_arg.to_string());
                res.push_str_method(name, new_arg);
            });
        res.push_doc_comment(attrs, "about");
        res.push_attrs(attrs);
        if res.has_custom_parser {
            span_error!(
                res.parser.span(),
                "parse attribute is only allowed on fields"
            );
        }
        match &*res.kind {
            Kind::Subcommand(_) => {
                span_error!(res.kind.span(), "subcommand is only allowed on fields")
            }
            Kind::FlattenStruct => {
                span_error!(res.kind.span(), "flatten is only allowed on fields")
            }
            Kind::Skip => span_error!(res.kind.span(), "skip is only allowed on fields"),
            Kind::Arg(_) => res,
        }
    }
    fn ty_from_field(ty: &syn::Type) -> Sp<Ty> {
        let t = |kind| Sp::new(kind, ty.span());
        if let syn::Type::Path(syn::TypePath {
            path: syn::Path { ref segments, .. },
            ..
        }) = *ty
        {
            match segments.iter().last().unwrap().ident.to_string().as_str() {
                "bool" => t(Ty::Bool),
                "Option" => derives::sub_type(ty)
                    .map(Attrs::ty_from_field)
                    .map(|ty| match *ty {
                        Ty::Option => t(Ty::OptionOption),
                        Ty::Vec => t(Ty::OptionVec),
                        _ => t(Ty::Option),
                    })
                    .unwrap_or(t(Ty::Option)),

                "Vec" => t(Ty::Vec),
                _ => t(Ty::Other),
            }
        } else {
            t(Ty::Other)
        }
    }
    pub fn from_field(field: &syn::Field, struct_casing: Sp<CasingStyle>) -> Self {
        let name = field.ident.clone().unwrap();
        let mut res = Self::new(name.into(), struct_casing);
        res.push_doc_comment(&field.attrs, "help");
        res.push_attrs(&field.attrs);

        match &*res.kind {
            Kind::FlattenStruct => {
                if res.has_custom_parser {
                    span_error!(
                        res.parser.span(),
                        "parse attribute is not allowed for flattened entry"
                    );
                }
                if !res.methods.is_empty() {
                    span_error!(
                        res.kind.span(),
                        "methods and doc comments are not allowed for flattened entry"
                    );
                }
            }
            Kind::Subcommand(_) => {
                if res.has_custom_parser {
                    span_error!(
                        res.parser.span(),
                        "parse attribute is not allowed for subcommand"
                    );
                }
                if let Some(m) = res.methods.iter().find(|m| m.name != "help") {
                    span_error!(
                        m.name.span(),
                        "methods in attributes are not allowed for subcommand"
                    );
                }

                let ty = Self::ty_from_field(&field.ty);
                match *ty {
                    Ty::OptionOption => {
                        span_error!(
                            ty.span(),
                            "Option<Option<T>> type is not allowed for subcommand"
                        );
                    }
                    Ty::OptionVec => {
                        span_error!(
                            ty.span(),
                            "Option<Vec<T>> type is not allowed for subcommand"
                        );
                    }
                    _ => (),
                }

                res.kind = Sp::new(Kind::Subcommand(ty), res.kind.span());
            }
            Kind::Skip => {
                if let Some(m) = res.methods.iter().find(|m| m.name != "help") {
                    span_error!(m.name.span(), "methods are not allowed for skipped fields");
                }
            }
            Kind::Arg(_) => {
                let mut ty = Self::ty_from_field(&field.ty);
                if res.has_custom_parser {
                    match *ty {
                        Ty::Option | Ty::Vec => (),
                        _ => ty = Sp::new(Ty::Other, ty.span()),
                    }
                }

                match *ty {
                    Ty::Bool => {
                        if let Some(m) = res.find_method("default_value") {
                            span_error!(m.name.span(), "default_value is meaningless for bool")
                        }
                        if let Some(m) = res.find_method("required") {
                            span_error!(m.name.span(), "required is meaningless for bool")
                        }
                    }
                    Ty::Option => {
                        if let Some(m) = res.find_method("default_value") {
                            span_error!(m.name.span(), "default_value is meaningless for Option")
                        }
                        if let Some(m) = res.find_method("required") {
                            span_error!(m.name.span(), "required is meaningless for Option")
                        }
                    }
                    Ty::OptionOption => {
                        // If it's a positional argument.
                        if !(res.has_method("long") || res.has_method("short")) {
                            span_error!(
                                ty.span(),
                                "Option<Option<T>> type is meaningless for positional argument"
                            )
                        }
                    }
                    Ty::OptionVec => {
                        // If it's a positional argument.
                        if !(res.has_method("long") || res.has_method("short")) {
                            span_error!(
                                ty.span(),
                                "Option<Vec<T>> type is meaningless for positional argument"
                            )
                        }
                    }

                    _ => (),
                }
                res.kind = Sp::call_site(Kind::Arg(ty));
            }
        }

        res
    }

    fn set_kind(&mut self, kind: Sp<Kind>) {
        if let Kind::Arg(_) = *self.kind {
            self.kind = kind;
        } else {
            span_error!(
                kind.span(),
                "subcommand, flatten and skip cannot be used together"
            );
        }
    }

    pub fn has_method(&self, name: &str) -> bool {
        self.find_method(name).is_some()
    }

    pub fn find_method(&self, name: &str) -> Option<&Method> {
        self.methods.iter().find(|m| m.name == name)
    }

    pub fn methods(&self) -> proc_macro2::TokenStream {
        let methods = self
            .methods
            .iter()
            .map(|&Method { ref name, ref args }| {
                if name == "short" {
                    quote!( .#name(#args.chars().nth(0).unwrap()) )
                } else {
                    quote!( .#name(#args) )
                }
            });

        quote!( #(#methods)* )
    }

    pub fn cased_name(&self) -> &str {
        &self.cased_name
    }

    pub fn parser(&self) -> &(Sp<Parser>, proc_macro2::TokenStream) {
        &self.parser
    }

    pub fn kind(&self) -> Sp<Kind> {
        self.kind.clone()
    }

    pub fn casing(&self) -> Sp<CasingStyle> {
        self.casing.clone()
    }
}
