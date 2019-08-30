use clap::Clap;

#[derive(Clap, Debug)]
#[clap(name = "test")]
pub struct Opts {
    #[clap(long)]
    a: u32,
    #[clap(skip, long)]
    b: u32,
}

fn main() {}
