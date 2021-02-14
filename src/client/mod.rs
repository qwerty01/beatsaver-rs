//! # Client
//!
//! This module contains client backend implmeentations.
//!
//! The following backends are implemented:
//! * [Reqwest](https://crates.io/crates/reqwest) => `reqwest_backend` feature (asynchronous, uses [Tokio](https://crates.io/crates/tokio))
//! * [Surf](https://crates.io/crates/surf) => `surf_backend` feature (asynchronous, uses [async-std](https://crates.io/crates/async-std))
//! * [ureq](https://crates.io/crates/ureq) => `ureq_backend` feature (synchronous)
//!
//! If only one backend is specified, it will be aliased to `BeatSaver`

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[cfg(feature = "reqwest_backend")]
mod reqwest_client;
#[cfg(feature = "reqwest_backend")]
pub use reqwest_client::BeatSaverReqwest;
#[cfg(all(
    feature = "reqwest_backend",
    not(feature = "surf_backend"),
    not(feature = "ureq_backend")
))]
pub use reqwest_client::BeatSaverReqwest as BeatSaver;

#[cfg(feature = "surf_backend")]
mod surf_client;
#[cfg(feature = "surf_backend")]
pub use surf_client::BeatSaverSurf;
#[cfg(all(
    feature = "surf_backend",
    not(feature = "reqwest_backend"),
    not(feature = "ureq_backend")
))]
pub use surf_client::BeatSaverSurf as BeatSaver;

#[cfg(feature = "ureq_backend")]
mod ureq_client;
#[cfg(feature = "ureq_backend")]
pub use ureq_client::BeatSaverUreq;
#[cfg(all(
    feature = "ureq_backend",
    not(feature = "reqwest_backend"),
    not(feature = "surf_backend")
))]
pub use ureq_client::BeatSaverUreq as BeatSaver;


