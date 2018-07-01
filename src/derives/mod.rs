// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>,
// Andrew Hobden (@hoverbear) <andrew@hoverbear.org>, and
// Kevin Knapp (@kbknapp) <kbknapp@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub mod arg_enum;
pub mod attrs;
pub mod clap;
pub mod from_argmatches;
pub mod into_app;
pub mod parse;

pub use self::arg_enum::impl_arg_enum;
pub use self::clap::impl_clap;
pub use self::from_argmatches::impl_from_argmatches;
pub use self::into_app::impl_into_app;
pub use self::parse::impl_parse;

use syn;

pub(crate) fn sub_type(t: &syn::Type) -> Option<&syn::Type> {
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
