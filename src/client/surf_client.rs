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

#[cfg(test)]
mod tests {
    //#[cfg(feature = "surf_backend")]
    #[async_std::test]
    async fn test_surf_map() {
        use crate::client::BeatSaverSurf;
        use crate::BeatSaverApiAsync;
        use std::convert::TryInto;

        let client = BeatSaverSurf::new();
        let map = client.map(&"2144".try_into().unwrap()).await.unwrap();

        assert_eq!(map.key, "2144");
    }
}