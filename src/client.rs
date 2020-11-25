#[cfg(feature = "reqwest_backend")]
mod reqwest_client {
    use reqwest::Client;
    use crate::{BeatSyncApiAsync, BeatSyncApiError};
    use async_trait::async_trait;
    use url::Url;
    use std::convert::From;
    use bytes::Bytes;

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
    impl<'a> BeatSyncApiAsync<'a, reqwest::Error> for BeatSyncReqwest {
        async fn request_raw(&'a self, url: Url) -> Result<Bytes, reqwest::Error> {
            self.client.get(url).send().await?.bytes().await
        }
    }
}
#[cfg(feature = "reqwest_backend")]
pub use reqwest_client::BeatSyncReqwest;
#[cfg(all(feature = "reqwest_backend", not(feature = "surf_backend"), not(feature = "ureq_backend")))]
pub use reqwest_client::BeatSyncReqwest as BeatSync;

#[cfg(feature = "surf_backend")]
mod surf_client {
    use surf::Client;
    use crate::{BeatSyncApiAsync, BeatSyncApiError};
    use async_trait::async_trait;
    use url::Url;
    use std::convert::From;
    use std::fmt::{self, Display, Formatter};
    use std::error::Error;
    use bytes::Bytes;
    
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
    impl<'a> BeatSyncApiAsync<'a, SurfError> for BeatSyncSurf {
        async fn request_raw(&'a self, url: Url) -> Result<Bytes, SurfError> {
            Ok(self.client.get(url).recv_bytes().await?.into())
        }
    }
}
#[cfg(feature = "surf_backend")]
pub use surf_client::BeatSyncSurf;
#[cfg(all(feature = "surf_backend", not(feature = "reqwest_backend"), not(feature = "ureq_backend")))]
pub use surf_client::BeatSyncSurf as BeatSync;

#[cfg(feature = "ureq_backend")]
mod ureq_client {
    use ureq;
    use crate::{BeatSyncApiSync, BeatSyncApiError};
    use url::Url;
    use std::convert::From;
    use bytes::Bytes;
    use std::io::Read;

    impl From<ureq::Error> for BeatSyncApiError<ureq::Error> {
        fn from(e: ureq::Error) -> Self {
            Self::RequestError(e)
        }
    }
    
    #[derive(Debug)]
    pub struct BeatSyncUreq {
    }
    impl BeatSyncUreq {
        // TODO: Allow user to specify client
        pub fn new() -> Self {
            Self { }
        }
    }
    impl<'a> BeatSyncApiSync<'a, ureq::Error> for BeatSyncUreq {
        fn request_raw(&'a self, url: Url) -> Result<Bytes, ureq::Error> {
            let mut contents = vec![];
            let resp = ureq::get(url.as_str()).call();
            let mut reader = resp.into_reader();
            reader.read_to_end(&mut contents)?;

            Ok(contents.into())
        }
    }
}
#[cfg(feature = "ureq_backend")]
pub use ureq_client::BeatSyncUreq;
#[cfg(all(feature = "ureq_backend", not(feature = "reqwest_backend"), not(feature = "surf_backend")))]
pub use ureq_client::BeatSyncUreq as BeatSync;

#[cfg(test)]
mod tests {
    #[cfg(feature = "surf_backend")]
    #[async_std::test]
    async fn test_surf_map() {
        use crate::client::BeatSyncSurf;
        use crate::BeatSyncApiAsync;
        use std::convert::TryInto;

        let client = BeatSyncSurf::new();
        let map = client.map(&"2144".try_into().unwrap()).await.unwrap();

        assert_eq!(map.key, "2144");
    }
    #[cfg(feature = "reqwest_backend")]
    #[tokio::test]
    async fn test_reqwest_map() {
        use crate::client::BeatSyncReqwest;
        use crate::BeatSyncApiAsync;
        use std::convert::TryInto;

        let client = BeatSyncReqwest::new();
        let map = client.map(&"2144".try_into().unwrap()).await.unwrap();

        assert_eq!(map.key, "2144");
    }
    #[cfg(feature = "ureq_backend")]
    #[test]
    fn test_ureq_map() {
        use crate::client::BeatSyncUreq;
        use crate::BeatSyncApiSync;
        use std::convert::TryInto;

        let client = BeatSyncUreq::new();
        let map = client.map(&"2144".try_into().unwrap()).unwrap();

        assert_eq!(map.key, "2144");
    }
}