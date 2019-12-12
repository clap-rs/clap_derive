//! How to use `clap::Arg::group`
#[macro_use]
extern crate clap;

use clap::{Clap, ArgGroup};

#[derive(Clap, Debug)]
#[clap(group = ArgGroup::with_name("verb").required(true))]
struct Opt {
    /// Set a custom HTTP verb
    #[clap(long, group = "verb")]
    method: Option<String>,
    /// HTTP GET
    #[clap(long, group = "verb")]
    get: bool,
    /// HTTP HEAD
    #[clap(long, group = "verb")]
    head: bool,
    /// HTTP POST
    #[clap(long, group = "verb")]
    post: bool,
    /// HTTP PUT
    #[clap(long, group = "verb")]
    put: bool,
    /// HTTP DELETE
    #[clap(long, group = "verb")]
    delete: bool,
}

fn main() {
    let opt = Opt::parse();
    println!("{:?}", opt);
}
