#![feature(box_syntax, test, fmt_internals)]
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
#[macro_use]
extern crate clap;

use clap::{App, Clap, Parse};

#[derive(Clap)]
#[clap(name = "myapp", version = "1.0")]
/// A basic example of a CLI for myapp
struct MyApp {
    /// Activate debug mode
    #[clap(short = "d", long = "debug")]
    debug: bool,

    /// Verbose mode
    #[clap(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u64,

    /// Set speed
    #[clap(short = "s", long = "speed", default_value = "42")]
    speed: f64,

    /// Output file
    #[clap(short = "o", long = "output")]
    output: String,

    /// Number of car
    #[clap(short = "c", long = "car")]
    car: Option<i32>,

    /// Files to process
    #[clap(name = "FILE")]
    files: Vec<String>,
}

#[test]
fn basic_clap() {
    let app = <MyApp as Parse>::parse_from(vec!["myapp", "-vvv", "--speed=20", "some", "files"]);

    assert!(!app.debug);
    assert_eq!(app.verbose, 3);
    assert_eq!(app.speed, 20.0);
    assert_eq!(app.files, &["some", "files"]);
}
