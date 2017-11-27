// Copyright ⓒ 2017 `clap-derive` Authors
//
// `clap-derive` is dual licensed under either of
//  * Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
//  * MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
// at your option. 

//! foo

#![crate_type= "lib"]
#![deny(
        missing_docs,
        missing_debug_implementations,
        missing_copy_implementations,
        trivial_casts,
        unused_import_braces,
        unused_allocation,
        unused_qualifications,       
        trivial_numeric_casts)]      
#![cfg_attr(not(any(feature = "lints", feature = "nightly")), forbid(unstable_features))]
#![cfg_attr(feature = "lints", feature(plugin))]
#![cfg_attr(feature = "lints", plugin(clippy))]
#![cfg_attr(feature = "lints", deny(warnings))]

extern crate clap;

use std::ffi::OsString;

/// @TODO @release @docs
pub trait ClapApp: IntoApp + FromArgMatches + Sized {

    /// @TODO @release @docs
    fn parse() -> Self {
        Self::from_argmatches(Self::into_app().get_matches())
    }

    /// @TODO @release @docs
    fn parse_from<I, T>(argv: I) -> Self
        where I: IntoIterator<Item = T>,
              T: Into<OsString> + Clone
    {
        Self::from_argmatches(Self::into_app().get_matches_from(argv))
    }


    /// @TODO @release @docs
    fn try_parse() -> Result<Self, clap::Error> {
        Self::try_from_argmatches(Self::into_app().get_matches_safe()?)
    }


    /// @TODO @release @docs
    fn try_parse_from<I, T>(argv: I) -> Result<Self, clap::Error> 
        where I: IntoIterator<Item = T>,
              T: Into<OsString> + Clone
    {
        Self::try_from_argmatches(Self::into_app().get_matches_from_safe(argv)?)
    }
}

/// @TODO @release @docs
pub trait IntoApp {
    /// @TODO @release @docs
    fn into_app<'a, 'b>() -> clap::App<'a, 'b>;
}

/// @TODO @release @docs
pub trait FromArgMatches: Sized {
    /// @TODO @release @docs
    fn from_argmatches<'a>(matches: clap::ArgMatches<'a>) -> Self;
    /// @TODO @release @docs
    fn try_from_argmatches<'a>(matches: clap::ArgMatches<'a>) -> Result<Self, clap::Error>;
}

/// @TODO @release @docs
pub trait ArgEnum { }