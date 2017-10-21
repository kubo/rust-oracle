# Rust Oracle - Work in progress

This is an Oracle driver for [Rust][] based on [ODPI-C][].

It is under development. Public API may be changed for each commit.

## Build-time Requirements

* Rust 1.18 or later
* C compiler. See `Compile-time Requirements` in [this document](https://docs.rs/crate/gcc/).

## Run-time Requirements

* Oracle Client 11.2 or later. See [ODPI-C installation document][].

## Usage

Rust Oracle has not been published to [crate.io](https://crates.io/).
You need to put this in your Cargo.toml:

```text
[dependencies]
oracle = { git = "https://github.com/kubo/rust-oracle.git" }
```

## License

Rust Oracle itself is under [2-clause BSD-style license](https://opensource.org/licenses/BSD-2-Clause).

ODPI-C bundled in Rust Oracle is under the terms of:

1. [the Universal Permissive License v 1.0 or at your option, any later version](http://oss.oracle.com/licenses/upl); and/or
2. [the Apache License v 2.0](http://www.apache.org/licenses/LICENSE-2.0). 

[Rust]:                 https://www.rust-lang.org/
[ODPI-C]:               https://oracle.github.io/odpi/
[ODPI-C installation document]: https://oracle.github.io/odpi/doc/installation.html
