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
mod reqwest_client {
    use super::USER_AGENT;
    use crate::{rate_limit, BeatSaverApiAsync, BeatSaverApiError};
    use async_trait::async_trait;
    use bytes::Bytes;
    use reqwest::Client;
    use reqwest::StatusCode;
    use std::convert::From;
    use url::Url;

    /// [BeatSaverApi][crate::BeatSaverApiAsync] implemented for [Reqwest][reqwest]
    #[derive(Debug, Clone)]
    pub struct BeatSaverReqwest {
        client: Client,
    }
    impl BeatSaverReqwest {
        /// Creates a new [BeatSaverReqwest][crate::client::BeatSaverReqwest] object, initiailizing a [Reqwest Client][reqwest::Client]
        ///
        /// Example:
        /// ```no_run
        /// use beatsaver_rs::client::BeatSaverReqwest;
        ///
        /// let client = BeatSaverReqwest::new();
        /// ```
        // TODO: Allow user to specify client
        pub fn new() -> Self {
            let client = Client::builder().user_agent(USER_AGENT).build().unwrap();
            Self { client }
        }
    }
    impl From<Client> for BeatSaverReqwest {
        fn from(client: Client) -> Self {
            Self { client }
        }
    }
    impl From<reqwest::Error> for BeatSaverApiError<reqwest::Error> {
        fn from(e: reqwest::Error) -> Self {
            Self::RequestError(e)
        }
    }
    #[async_trait]
    impl<'a> BeatSaverApiAsync<'a, reqwest::Error> for BeatSaverReqwest {
        async fn request_raw(
            &'a self,
            url: Url,
        ) -> Result<Bytes, BeatSaverApiError<reqwest::Error>> {
            let resp = self.client.get(url).send().await?;
            let status = resp.status();
            let data = resp.bytes().await?;

            match status {
                StatusCode::TOO_MANY_REQUESTS => Err(rate_limit(data)),
                _ => Ok(data),
            }
        }
    }
}
#[cfg(feature = "reqwest_backend")]
pub use reqwest_client::BeatSaverReqwest;
#[cfg(all(
    feature = "reqwest_backend",
    not(feature = "surf_backend"),
    not(feature = "ureq_backend")
))]
pub use reqwest_client::BeatSaverReqwest as BeatSaver;

#[cfg(feature = "surf_backend")]
mod surf_client {
    use super::USER_AGENT;
    use crate::{rate_limit, BeatSaverApiAsync, BeatSaverApiError};
    use async_trait::async_trait;
    use bytes::Bytes;
    use std::convert::From;
    use std::error::Error;
    use std::fmt::{self, Display, Formatter};
    use surf::{Client, StatusCode};
    use url::Url;

    /// [Error][std::error::Error] wrapper type for [surf::Error]
    #[derive(Debug)]
    pub enum SurfError {
        /// Surf error
        Error(surf::Error),
    }
    impl Display for SurfError {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            match self {
                Self::Error(e) => e.fmt(f),
            }
        }
    }
    impl Error for SurfError {}
    impl From<surf::Error> for SurfError {
        fn from(e: surf::Error) -> Self {
            Self::Error(e)
        }
    }
    impl From<SurfError> for BeatSaverApiError<SurfError> {
        fn from(e: SurfError) -> Self {
            Self::RequestError(e)
        }
    }
    impl From<surf::Error> for BeatSaverApiError<SurfError> {
        fn from(e: surf::Error) -> Self {
            Self::RequestError(e.into())
        }
    }

    /// [BeatSaverApi][crate::BeatSaverApiAsync] implemented for [Surf][surf]
    #[derive(Debug, Clone)]
    pub struct BeatSaverSurf {
        client: Client,
    }
    impl BeatSaverSurf {
        /// Creates a new [BeatSaverSurf][crate::client::BeatSaverSurf] object, initiailizing a [Surf Client][surf::Client]
        ///
        /// Example:
        /// ```no_run
        /// use beatsaver_rs::client::BeatSaverSurf;
        ///
        /// let client = BeatSaverSurf::new();
        /// ```
        // TODO: Allow user to specify client
        pub fn new() -> Self {
            let client = Client::new();
            Self { client }
        }
    }
    impl From<Client> for BeatSaverSurf {
        fn from(client: Client) -> Self {
            Self { client }
        }
    }
    #[async_trait]
    impl<'a> BeatSaverApiAsync<'a, SurfError> for BeatSaverSurf {
        async fn request_raw(&'a self, url: Url) -> Result<Bytes, BeatSaverApiError<SurfError>> {
            let mut resp = self
                .client
                .get(url)
                .header("User-Agent", USER_AGENT)
                .await?;
            let data = resp.body_bytes().await?.into();
            match resp.status() {
                StatusCode::TooManyRequests => Err(rate_limit(data)),
                _ => Ok(data),
            }
        }
    }
}
#[cfg(feature = "surf_backend")]
pub use surf_client::BeatSaverSurf;
#[cfg(all(
    feature = "surf_backend",
    not(feature = "reqwest_backend"),
    not(feature = "ureq_backend")
))]
pub use surf_client::BeatSaverSurf as BeatSaver;

#[cfg(feature = "ureq_backend")]
mod ureq_client {
    use super::USER_AGENT;
    use crate::{rate_limit, BeatSaverApiError, BeatSaverApiSync};
    use bytes::Bytes;
    use std::convert::From;
    use std::io::Read;
    use ureq;
    use url::Url;

    impl From<ureq::Error> for BeatSaverApiError<ureq::Error> {
        fn from(e: ureq::Error) -> Self {
            Self::RequestError(e)
        }
    }

    /// [BeatSaverApi][crate::BeatSaverApiSync] implemented for [ureq]
    #[derive(Debug)]
    pub struct BeatSaverUreq {}
    impl BeatSaverUreq {
        /// Creates a new [BeatSaverUreq][crate::client::BeatSaverUreq] object
        ///
        /// Example:
        /// ```no_run
        /// use beatsaver_rs::client::BeatSaverUreq;
        ///
        /// let client = BeatSaverUreq::new();
        /// ```
        // TODO: Allow user to specify client
        pub fn new() -> Self {
            Self {}
        }
    }
    impl<'a> BeatSaverApiSync<'a, ureq::Error> for BeatSaverUreq {
        fn request_raw(&'a self, url: Url) -> Result<Bytes, BeatSaverApiError<ureq::Error>> {
            let mut contents = vec![];
            match ureq::get(url.as_str()).set("User-Agent", USER_AGENT).call() {
                Ok(resp) => {
                    let mut reader = resp.into_reader();
                    reader.read_to_end(&mut contents)?;
                    Ok(contents.into())
                }
                Err(ureq::Error::Status(code, resp)) => {
                    let mut reader = resp.into_reader();
                    reader.read_to_end(&mut contents)?;
                    match code {
                        429 => Err(rate_limit(contents.into())),
                        // TODO: req doesn't have an error type for HTTP errors, might need
                        // to do some extra checks with the http crate in the future
                        _ => Ok(contents.into()),
                    }
                }
                Err(e) => {
                    Err(e.into())
                }
            }
        }
    }
}
#[cfg(feature = "ureq_backend")]
pub use ureq_client::BeatSaverUreq;
#[cfg(all(
    feature = "ureq_backend",
    not(feature = "reqwest_backend"),
    not(feature = "surf_backend")
))]
pub use ureq_client::BeatSaverUreq as BeatSaver;

#[cfg(test)]
mod tests {
    #[cfg(feature = "surf_backend")]
    #[async_std::test]
    async fn test_surf_map() {
        use crate::client::BeatSaverSurf;
        use crate::BeatSaverApiAsync;
        use std::convert::TryInto;

        let client = BeatSaverSurf::new();
        let map = client.map(&"2144".try_into().unwrap()).await.unwrap();

        assert_eq!(map.key, "2144");
    }
    #[cfg(feature = "reqwest_backend")]
    #[tokio::test]
    async fn test_reqwest_map() {
        use crate::client::BeatSaverReqwest;
        use crate::BeatSaverApiAsync;
        use std::convert::TryInto;

        let client = BeatSaverReqwest::new();
        let map = client.map(&"2144".try_into().unwrap()).await.unwrap();

        assert_eq!(map.key, "2144");
    }
    #[cfg(feature = "ureq_backend")]
    #[test]
    fn test_ureq_map() {
        use crate::client::BeatSaverUreq;
        use crate::BeatSaverApiSync;
        use std::convert::TryInto;

        let client = BeatSaverUreq::new();
        let map = client.map(&"2144".try_into().unwrap()).unwrap();

        assert_eq!(map.key, "2144");
    }
}
