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

/// Used for Custom Derive (in the `clap_derive` crate) to automatically build an `App` from an
/// arbitrary struct, and then deserialize the argument matches back into that struct automatically
pub trait ClapApp: IntoApp + FromArgMatches {
    /// Gets the struct from the command line arguments.  Print the
    /// error message and quit the program in case of failure.
    fn parse() -> Self where Self: Sized {
        Self::from_clap(Self::clap().get_matches())
    }
    /// Gets the struct from the command line arguments.  Print the
    /// error message and quit the program in case of failure.
    fn try_parse(v: Vec<_>) -> Result<Self, _> where Self: Sized {
        Self::from_clap(Self::clap().get_matches())
    }
    /// Gets the struct from the command line arguments.  Print the
    /// error message and quit the program in case of failure.
    fn parse_from(v: Vec<_>) -> Self where Self: Sized {
        Self::from_clap(Self::clap().get_matches())
    }
    /// Gets the struct from the command line arguments.  Print the
    /// error message and quit the program in case of failure.
    fn try_parse_from(v: Vec<_>) -> Result<Self, _> where Self: Sized {
        Self::from_clap(Self::clap().get_matches())
    }
}

/// Used for Custom Derive (in the `clap_derive` crate) to automatically build an `App` from an
/// arbitrary struct
pub trait IntoApp {
    /// Returns the corresponding `clap::App`.
    fn into_app<'a, 'b>() -> clap::App<'a, 'b>;
}
