# beatsaver-rs

This project is a Rust library for interacting with the beatsaver.com api.

## Installation

```bash
cargo install beatsaver-rs
```

## Usage

API has not been stabalized yet.

## Async Runtimes

Currently, this crate supports two async backends: [`reqwest`](https://crates.io/crates/reqwest) which runs on [`tokio`](https://crates.io/crates/tokio) and [`surf`](https://crates.io/crates/surf) which runs on [`async-std`](https://crates.io/crates/async-std). By default, [`reqwest`](https://crates.io/crates/reqwest) is used, but you can specify a particular backend by enabling the `[backend]_backend` feature (for example, `surf_backend`).

When testing, make sure to enable all features to ensure all backends are tested properly:

```bash
cargo test --all-features
```

## License
[MIT](LICENSE)