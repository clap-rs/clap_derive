#[macro_use]
extern crate clap;

use clap::Clap;

#[test]
fn explicit_short_long_no_rename() {
    #[derive(Clap, PartialEq, Debug)]
    struct Opt {
        #[clap(short = ".", long = ".foo")]
        foo: Vec<String>,
    }

    assert_eq!(
        Opt {
            foo: vec!["short".into(), "long".into()]
        },
        Opt::parse_from(&["test", "-.", "short", "--.foo", "long"])
    );
}

#[test]
fn explicit_name_no_rename() {
    use clap::IntoApp;

    #[derive(Clap, PartialEq, Debug)]
    struct Opt {
        #[clap(name = ".options")]
        foo: Vec<String>,
    }

    let mut output = Vec::new();
    Opt::into_app().write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("[.options]..."))
}
