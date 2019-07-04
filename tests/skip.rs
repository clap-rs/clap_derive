// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate clap;

use clap::Clap;

#[test]
fn skip_1() {
    #[derive(Clap, Debug, PartialEq)]
    struct Opt {
        #[clap(short)]
        x: u32,
        #[clap(skip)]
        s: u32,
    }

    assert!(Opt::try_parse_from(&["test", "-x", "10", "20"]).is_err());
    assert_eq!(
        Opt::parse_from(&["test", "-x", "10"]),
        Opt {
            x: 10,
            s: 0, // default
        }
    );
}

#[test]
fn skip_2() {
    #[derive(Clap, Debug, PartialEq)]
    struct Opt {
        #[clap(short)]
        x: u32,
        #[clap(skip)]
        ss: String,
        #[clap(skip)]
        sn: u8,
        y: u32,
        #[clap(skip)]
        sz: u16,
        t: u32,
    }

    assert_eq!(
        Opt::parse_from(&["test", "-x", "10", "20", "30"]),
        Opt {
            x: 10,
            ss: String::from(""),
            sn: 0,
            y: 20,
            sz: 0,
            t: 30,
        }
    );
}

#[test]
fn skip_enum() {
    #[derive(Debug, PartialEq)]
    #[allow(unused)]
    enum Kind {
        A,
        B,
    }

    impl Default for Kind {
        fn default() -> Self {
            return Kind::B;
        }
    }

    #[derive(Clap, Debug, PartialEq)]
    #[clap(name = "a")]
    pub struct Opt {
        #[clap(long, short)]
        number: u32,
        #[clap(skip)]
        k: Kind,
        #[clap(skip)]
        v: Vec<u32>,
    }

    assert_eq!(
        Opt::parse_from(&["test", "-n", "10"]),
        Opt {
            number: 10,
            k: Kind::B,
            v: vec![],
        }
    );
}
