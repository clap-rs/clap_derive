extern crate clap;
#[macro_use]
extern crate clap_derive;

use clap::{App, Arg, ArgEnum};

// @TODO @tests @p4: check hyphen values like ArgChoice::Foo_Bar from foo-bar

#[derive(ArgEnum, Debug)]
enum ArgChoice {
    Foo,
    Bar,
    Baz,
}

#[test]
fn argenum() {
    let matches = App::new("foo")
            .arg(Arg::with_name("arg")
                .required(true)
                .takes_value(true)
                .possible_values(&ArgChoice::variants())
            ).get_matches_from(vec!["foo", "Bar"]);
    
    let t = matches.value_of("arg").unwrap().parse::<ArgChoice>()
        .unwrap_or_else(|e| e.exit());

    assert_eq!(t, ArgChoice::Bar);
}

#[test]
fn argenum_case_insensitive() {
    let matches = App::new("foo")
            .arg(Arg::with_name("arg")
                .required(true)
                .takes_value(true)
                .possible_values(&ArgChoice::variants())
            ).get_matches_from(vec!["foo", "bar"]);
    
    let t = matches.value_of("arg").unwrap().parse::<ArgChoice>()
        .unwrap_or_else(|e| e.exit());

    assert_eq!(t, ArgChoice::Bar);
}