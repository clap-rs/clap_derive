//! How to require presence of at least N values,
//! like `val1 val2 ... valN ... valM`.

#[macro_use]
extern crate clap;

use clap::Clap;

#[derive(Clap, Debug)]
struct Opt {
    #[clap(required = true, min_values = 2)]
    foos: Vec<String>,
}

fn main() {
    let opt = Opt::parse();
    println!("{:?}", opt);
}
