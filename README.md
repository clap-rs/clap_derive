# clap-derives

[![Build Status](https://travis-ci.org/kbknapp/clap-derive.svg?branch=master)](https://travis-ci.org/kbknapp/clap-derive)
[![Build status](https://ci.appveyor.com/api/projects/status/w8v2poyjwsy5d05k?svg=true)](https://ci.appveyor.com/project/kbknapp/clap-derive)

Clap custom derives.

```rust
#[macro_use]
extern crate clap;
#[macro_use]
extern crate clap_derive;

use clap::{App, Arg};

#[derive(ArgEnum, Debug)]
enum ArgChoice {
    Foo,
    Bar,
    Baz,
}

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
            .arg(Arg::with_name("arg")
                .required(true)
                .takes_value(true)
                .possible_values(&ArgChoice::variants())
            ).get_matches();
    
    let t = value_t!(matches.value_of("arg"), ArgChoice)
        .unwrap_or_else(|e| e.exit());

    println!("{:?}", t);
}
```

## License

Copyright ⓒ 2017 `clap-derive` Authors

`clap-derive` is dual licensed under either of

 * Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option. Unless specifcally stated otherwise, all contributes will be licensed as such.