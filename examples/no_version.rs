#[macro_use]
extern crate clap;

use clap::{AppSettings, Clap};

#[derive(Clap, Debug)]
#[clap(
    name = "no_version",
    about = "",
    version = "",
    author = "",
    global_setting = AppSettings::DisableVersion
)]
struct Opt {}

fn main() {
    let opt = Opt::parse();
    println!("{:?}", opt);
}
