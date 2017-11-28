use std::env;

use syn;

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum Ty {
    Bool,
    U64,
    Vec,
    Option,
    Other,
}

pub(crate) fn ty(t: &syn::Ty) -> Ty {
    if let syn::Ty::Path(
        None,
        syn::Path {
            segments: ref segs, ..
        },
    ) = *t
    {
        match segs.last().unwrap().ident.as_ref() {
            "bool" => Ty::Bool,
            "u64" => Ty::U64,
            "Option" => Ty::Option,
            "Vec" => Ty::Vec,
            _ => Ty::Other,
        }
    } else {
        Ty::Other
    }
}

pub(crate) fn sub_type(t: &syn::Ty) -> Option<&syn::Ty> {
    let segs = match *t {
        syn::Ty::Path(None, syn::Path { ref segments, .. }) => segments,
        _ => return None,
    };
    match *segs.last().unwrap() {
        syn::PathSegment {
            parameters:
                syn::PathParameters::AngleBracketed(syn::AngleBracketedParameterData { ref types, .. }),
            ..
        } if !types.is_empty() =>
        {
            Some(&types[0])
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AttrSource {
    Struct,
    Field,
}

#[derive(Debug)]
pub(crate) enum Parser {
    /// Parse an option to using a `fn(&str) -> T` function. The function should never fail.
    FromStr,
    /// Parse an option to using a `fn(&str) -> Result<T, E>` function. The error will be
    /// converted to a string using `.to_string()`.
    TryFromStr,
    /// Parse an option to using a `fn(&OsStr) -> T` function. The function should never fail.
    FromOsStr,
    /// Parse an option to using a `fn(&OsStr) -> Result<T, OsString>` function.
    TryFromOsStr,
}

pub(crate) fn extract_attrs<'a>(
    attrs: &'a [syn::Attribute],
    attr_source: AttrSource,
) -> Box<Iterator<Item = (syn::Ident, syn::Lit)> + 'a> {
    let settings_attrs = attrs
        .iter()
        .filter_map(|attr| match attr.value {
            syn::MetaItem::List(ref i, ref v) if i.as_ref() == "clap" => Some(v),
            _ => None,
        })
        .flat_map(|v| {
            v.iter().filter_map(|mi| match *mi {
                syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref i, ref l)) => {
                    Some((i.clone(), l.clone()))
                }
                _ => None,
            })
        });

    let doc_comments: Vec<String> = attrs
        .iter()
        .filter_map(move |attr| {
            if let syn::Attribute {
                value:
                    syn::MetaItem::NameValue(ref name, syn::Lit::Str(ref value, syn::StrStyle::Cooked)),
                is_sugared_doc: true,
                ..
            } = *attr
            {
                if name != "doc" {
                    return None;
                }
                let text = value
                    .trim_left_matches("//!")
                    .trim_left_matches("///")
                    .trim_left_matches("/*!")
                    .trim_left_matches("/**")
                    .trim();
                Some(text.into())
            } else {
                None
            }
        })
        .collect();

    let doc_comments = if doc_comments.is_empty() {
        None
    } else if let AttrSource::Struct = attr_source {
        // Clap's `App` has an `about` method to set a description,
        // it's `Field`s have a `help` method instead.
        Some(("about".into(), doc_comments.join(" ").into()))
    } else {
        Some(("help".into(), doc_comments.join(" ").into()))
    };

    Box::new(doc_comments.into_iter().chain(settings_attrs))
}

pub(crate) fn from_attr_or_env(attrs: &[(syn::Ident, syn::Lit)], key: &str, env: &str) -> syn::Lit {
    let default = env::var(env).unwrap_or_default();
    attrs
        .iter()
        .filter(|&&(ref i, _)| i.as_ref() == key)
        .last()
        .map(|&(_, ref l)| l.clone())
        .unwrap_or_else(|| syn::Lit::Str(default, syn::StrStyle::Cooked))
}
