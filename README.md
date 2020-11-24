# beatsaver-rs

This project is a Rust library for interacting with the beatsaver.com api.

## Installation

```bash
cargo install beatsaver-rs
```

## Usage

API has not been stabalized yet.

## Async Runtimes

Currently, this crate supports two async runtimes: [`tokio`](https://crates.io/crates/tokio) and [`async-std`](https://crates.io/crates/async-std). By default, [`tokio`](https://crates.io/crates/tokio) is used, but you can specify a particular runtime by enabling the `[runtime]_runtime` feature (for example, `async-std_runtime`).

Because features cannot be modified by test code, in order to ensure code is compatible with both, tests need to be run with both features:

```bash
cargo test
cargo test --no-default-features --features async-std_runtime
```

## License
[MIT](LICENSE)