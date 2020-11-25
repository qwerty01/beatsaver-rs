#[cfg(feature = "reqwest_backend")]
mod reqwest_client {
    use reqwest::Client;
    use crate::{BeatSyncApi, BeatSyncApiError};
    use async_trait::async_trait;
    use url::Url;
    use std::convert::From;

    #[derive(Debug, Clone)]
    pub struct BeatSyncReqwest {
        client: Client
    }
    impl BeatSyncReqwest {
        // TODO: Allow user to specify client
        pub fn new() -> Self {
            let client = Client::builder().user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))).build().unwrap();
            Self { client }
        }
    }
    impl From<reqwest::Error> for BeatSyncApiError<reqwest::Error> {
        fn from(e: reqwest::Error) -> Self {
            Self::RequestError(e)
        }
    }
    #[async_trait]
    impl<'a> BeatSyncApi<'a, reqwest::Error> for BeatSyncReqwest {
        async fn request(&'a self, url: Url) -> Result<String, reqwest::Error> {
            self.client.get(url).send().await?.text().await
        }
    }
}

#[cfg(feature = "reqwest_backend")]
pub use reqwest_client::BeatSyncReqwest;
#[cfg(all(feature = "reqwest_backend", not(feature = "surf_backend"), not(feature = "curl")))]
pub use reqwest_client::BeatSyncReqwest as BeatSync;

#[cfg(feature = "surf_backend")]
mod surf_client {
    use surf::Client;
    use crate::{BeatSyncApi, BeatSyncApiError};
    use async_trait::async_trait;
    use url::Url;
    use std::convert::From;
    use std::fmt::{self, Display, Formatter};
    use std::error::Error;
    
    #[derive(Debug)]
    pub enum SurfError {
        Error(surf::Error)
    }
    impl Display for SurfError {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            match self {
                Self::Error(e) => e.fmt(f)
            }
        }
    }
    impl Error for SurfError {}
    impl From<surf::Error> for SurfError {
        fn from(e: surf::Error) -> Self {
            Self::Error(e)
        }
    }
    impl From<SurfError> for BeatSyncApiError<SurfError> {
        fn from(e: SurfError) -> Self {
            Self::RequestError(e)
        }
    }
    impl From<surf::Error> for BeatSyncApiError<SurfError> {
        fn from(e: surf::Error) -> Self {
            Self::RequestError(e.into())
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct BeatSyncSurf {
        client: Client
    }
    impl BeatSyncSurf {
        // TODO: Allow user to specify client
        pub fn new() -> Self {
            let client = Client::new();
            Self { client }
        }
    }
    #[async_trait]
    impl<'a> BeatSyncApi<'a, SurfError> for BeatSyncSurf {
        async fn request(&'a self, url: Url) -> Result<String, SurfError> {
            Ok(self.client.get(url).recv_string().await?)
        }
    }
}

#[cfg(feature = "surf_backend")]
pub use surf_client::BeatSyncSurf;
#[cfg(all(feature = "surf_backend", not(feature = "reqwest_backend"), not(feature = "curl")))]
pub use surf_client::BeatSyncSurf as BeatSync;

#[cfg(test)]
mod tests {
    #[cfg(feature = "surf_backend")]
    #[async_std::test]
    async fn test_surf_map() {
        use crate::client::BeatSyncSurf;
        use crate::BeatSyncApi;
        use std::convert::TryInto;

        let client = BeatSyncSurf::new();
        let map = client.map(&"2144".try_into().unwrap()).await.unwrap();

        assert_eq!(map.key, "2144");
    }
    #[cfg(feature = "reqwest_backend")]
    #[tokio::test]
    async fn test_reqwest_map() {
        use crate::client::BeatSyncReqwest;
        use crate::BeatSyncApi;
        use std::convert::TryInto;

        let client = BeatSyncReqwest::new();
        let map = client.map(&"2144".try_into().unwrap()).await.unwrap();

        assert_eq!(map.key, "2144");
    }
}