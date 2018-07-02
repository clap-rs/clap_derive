extern crate clap;
#[macro_use]
extern crate clap_derive;

use clap::{App, IntoApp};

#[derive(FromArgMatches, IntoApp)]
#[clap(name = "myapp", version = "1.0")]
/// A basic example of a CLI for myapp
struct MyApp {
    /// Activate debug mode
    #[clap(short = "d", long = "debug")]
    debug: bool,

    /// Verbose mode
    #[clap(short = "v", long = "verbose")]
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
fn from_argmatches() {
    let app: App = MyApp::empty().into();
    let myapp: MyApp = app.get_matches_from(vec!["myapp", "-v", "--speed=20", "some", "files"])
        .into();

    assert!(!myapp.debug);
    assert!(myapp.verbose);
    assert_eq!(myapp.speed, 20);
    assert_eq!(myapp.files, &["some", "files"]);
}
