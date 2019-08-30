// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use clap::Clap;

#[derive(Debug)]
enum Kind {
    A,
    B,
}

#[derive(Clap, Debug)]
#[clap(name = "test")]
pub struct Opts {
    #[clap(short)]
    number: u32,
    #[clap(skip)]
    k: Kind,
}

fn main() {}
