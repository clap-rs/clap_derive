use std::num::ParseIntError;
use std::path::PathBuf;

// @TODO @tests @p4: should add tests for default fn's too
// @TODO @tests @p3: should assert error kind in tests

// parses hex
fn custom_try_from_str(src: &str) -> Result<u32, ParseIntError> {
    u32::from_str_radix(src, 16)
}

#[derive(ClapApp)]
struct CustomTryFromStr {
    #[clap(short = "n", parse_with(try_from_str = "custom_try_from_str"))]
    number: u32,
}

#[test]
fn custom_try_from_str() {
    let hr = CustomTryFromStr::parse_from(vec!["hexreader", "-n", "0x28"]);
    assert_eq!(hr.number, 40);
}

#[test]
fn custom_try_from_str_fail() {
    let res = CustomTryFromStr::try_parse_from(vec!["hexreader", "-n", "0xzz"]);
    assert!(res.is_err());
    // check error kind
}

// parses hex
fn custom_from_str(src: &str) -> u32 {
    u32::from_str_radix(src, 16).unwrap()
}

#[derive(ClapApp)]
struct CustomFromStr {
    #[clap(short = "n", parse_with(from_str = "custom_from_str"))]
    number: u32,
}

#[test]
fn custom_from_str() {
    let hr = CustomFromStr::parse_from(vec!["hexreader", "-n", "0x28"]);
    assert_eq!(hr.number, 40);
}

#[test]
fn custom_from_str_fail() {
    let res = CustomFromStr::try_parse_from(vec!["hexreader", "-n", "0xzz"]);
    assert!(res.is_err());
    // check error kind
}

// parses hex
fn custom_try_from_os_str(src: &OsStr) -> Result<u32, ParseIntError> {
    u32::from_str_radix(src.to_str().unwrap(), 16)
}

#[derive(ClapApp)]
struct CustomTryFromOsStr {
    #[clap(short = "n", parse_with(try_from_os_str = "custom_try_from_os_str"))]
    number: u32,
}

#[test]
fn custom_try_from_os_str() {
    let hr = CustomTryFromOsStr::parse_from(vec!["hexreader", "-n", "0x28"]);
    assert_eq!(hr.number, 40);
}

#[test]
fn custom_try_from_os_str_fail() {
    let res = CustomTryFromOsStr::try_parse_from(vec!["hexreader", "-n", "0xzz"]);
    assert!(res.is_err());
    // check error kind
}

// parses hex
fn custom_from_os_str(src: &OsStr) -> u32 {
    u32::from_str_radix(src.to_str().unwrap(), 16).unwrap()
}

#[derive(ClapApp)]
struct CustomFromOsStr {
    #[clap(short = "n", parse_with(from_os_str = "custom_from_os_str"))]
    number: u32,
}

#[test]
fn custom_from_os_str() {
    let hr = CustomFromOsStr::parse_from(vec!["hexreader", "-n", "0x28"]);
    assert_eq!(hr.number, 40);
}

#[test]
fn custom_from_os_str_fail() {
    let res = CustomFromOsStr::try_parse_from(vec!["hexreader", "-n", "0xzz"]);
    assert!(res.is_err());
    // check error kind
}
