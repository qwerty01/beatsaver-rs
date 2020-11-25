# beatsaver-rs

[![Crates.io](https://img.shields.io/crates/v/beatsaver-rs.svg)](https://crates.io/crates/beatsaver-rs)
[![Documentation](https://docs.rs/beatsaver-rs/badge.svg)](https://docs.rs/beatsaver-rs)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub issues](https://img.shields.io/github/issues/qwerty01/beatsaver-rs)](https://github.com/qwerty01/beatsaver-rs/issues)
[![Build](https://github.com/qwerty01/beatsaver-rs/workflows/Build/badge.svg)](https://github.com/qwerty01/beatsaver-rs/actions?query=workflow%3ABuild)

This project is a Rust library for interacting with the beatsaver.com api.

## Installation

```bash
cargo install beatsaver-rs
```

## Usage

API has not been stabalized yet.

## Backends

Currently, this crate supports three backends:
* [`reqwest`](https://crates.io/crates/reqwest), which is asynchronous and runs on [`tokio`](https://crates.io/crates/tokio)
* [`surf`](https://crates.io/crates/surf), which is asynchronous and runs on [`async-std`](https://crates.io/crates/async-std)
* [`ureq`](https://crates.io/crates/ureq), which is synchronous.

By default, [`reqwest`](https://crates.io/crates/reqwest) is used, but you can specify a particular backend by enabling the `[backend]_backend` feature (for example, `surf_backend`).

## Testing

When testing, make sure to enable all features to ensure all backends are tested properly:

```bash
cargo test --all-features
```

## License
[MIT](LICENSE)