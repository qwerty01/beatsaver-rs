use super::USER_AGENT;
use crate::{rate_limit, BeatSaverApiAsync, BeatSaverApiError};
use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Client;
use reqwest::StatusCode;
use std::convert::From;
use url::Url;

/// [BeatSaverApi][crate::BeatSaverApiAsync] implemented for [Reqwest][reqwest]
#[derive(Clone, Debug, Default)]
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

#[cfg(test)]
mod tests {
    //#[cfg(feature = "reqwest_backend")]
    #[tokio::test]
    async fn test_reqwest_map() {
        use crate::client::BeatSaverReqwest;
        use crate::BeatSaverApiAsync;
        use std::convert::TryInto;

        let client = BeatSaverReqwest::new();
        let map = client.map(&"2144".try_into().unwrap()).await.unwrap();

        assert_eq!(map.key, "2144");
    }
}