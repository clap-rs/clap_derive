#[macro_use]
extern crate clap;

use clap::Clap;

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

fn main() {
    assert_eq!(
        Opt::parse_from(&["test", "-n", "10"]),
        Opt {
            number: 10,
            k: Kind::B,
            v: vec![],
        }
    );
}
