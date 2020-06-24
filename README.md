# cargo-nav

![Rust CI](https://github.com/Celeo/cargo-nav/workflows/Rust%20CI/badge.svg?branch=master)
![Crates.io](https://img.shields.io/crates/v/cargo-nav.svg)
![License](https://img.shields.io/crates/l/cargo-nav)

Navigate directly to crate links from your terminal.

Inspired by [njt](https://github.com/kachkaev/njt).

## Installing

```sh
cargo install cargo-nav
```

## Using

Get usage information with `cargo nav --help`

Call via `cargo nav <crate_name>` to jump to the homepage of that crate as listed on [crates.io](https://crates.io/). You can specify an additional argument to jump to the [r]epository, [d]ocumentation, or [c]rate pages.

```sh
cargo nav serde
cargo nav serde c
cargo nav serde crate
cargo nav serde h
cargo nav serde homepage
cargo nav serde r
cargo nav serde repository
cargo nav serde d
cargo nav serde documentation
```

The short arguments 'h', 'r', and 'd' are available as less typing to get to their respective links. Going to the crate's homepage is the default behavior.

## Developing

### Building

### Requirements

* Git
* A recent version of [Rust](https://www.rust-lang.org/tools/install)

### Steps

```sh
git clone https://github.com/Celeo/cargo-nav
cd cargo-nav
cargo build
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))

## Contributing

Please feel free to contribute. Please open an issue first (or comment on an existing one) so that I know that you want to add/change something.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
