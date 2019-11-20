#[macro_use]
extern crate clap;

use clap::Clap;

#[derive(Debug, Clap)]
struct Opt {
    #[clap(long = "no-verbose", parse(from_flag = std::ops::Not::not))]
    verbose: bool,
}

fn main() {
    let cmd = Opt::parse();
    println!("{:#?}", cmd);
}
