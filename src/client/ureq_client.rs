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

#[cfg(test)]
mod tests {
    //#[cfg(feature = "ureq_backend")]
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