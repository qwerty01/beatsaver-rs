#![warn(missing_docs)]
//! # beatsaver-rs
//!
//! This probject is a Rust library for interacting with the [BeatSaver](https://beatsaver.com/) api.
//!
//! The library is a work in progress and the API has not been stablized, so expect breaking changes.
//!
//! See also:
//! * [BeatSaver API Docs](https://docs.beatsaver.com/)
//!
//! # Using the API
//!
//! ```no_run
//! # #[cfg(all(feature = "reqwest_backend", not(feature = "surf_backend"), not(feature = "ureq_backend")))]
//! # mod main {
//! use beatsaver_rs::BeatSaverApi;
//! use beatsaver_rs::client::BeatSaver;
//! use beatsaver_rs::map::Map;
//! use bytes::Bytes;
//! use std::convert::TryInto;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = BeatSaver::new();
//!     let map: Map = client.map(&"1".try_into().unwrap()).await.unwrap();
//!     println!("Map by key: {}", map.name);
//!     let map: Map = client.map(&"fda568fc27c20d21f8dc6f3709b49b5cc96723be".try_into().unwrap()).await.unwrap();
//!     println!("Map by hash: {}", map.name);
//!     let map_download: Bytes = client.download((&map).into()).await.unwrap();
//!     // save map somewhere
//! }
//! # }
//! ```
use bytes::Bytes;
use chrono::{DateTime, NaiveDateTime, Utc};
use fmt::Debug;
use hex::{self, FromHexError};
use lazy_static::lazy_static;
use thiserror::Error;
use map::Map;
use serde::{de, Deserialize, Serialize};
use serde_json;
use std::{cmp::Ordering, collections::VecDeque, hash::{Hash, Hasher}};
use std::convert::{From, TryFrom, TryInto};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use std::time::Duration;
use url::Url;

mod async_api;
pub mod client;
pub mod map;
mod sync_api;

lazy_static! {
    /// Base URL for the beatsaver API
    pub static ref BEATSAVER_URL: Url = Url::parse("https://beatsaver.com/").unwrap();
}

/// Holds data for a beatsaver user
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeatSaverUser {
    /// User ID (e.g. `5fbe7cd60192c700062b2a1f`)
    #[serde(alias = "_id")]
    pub id: String,
    /// User name (e.g. `qwerty01`)
    pub username: String,
}
impl PartialEq for BeatSaverUser {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for BeatSaverUser {}
impl Hash for BeatSaverUser {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl Display for BeatSaverUser {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.username, f)
    }
}

/// Page metadata for APIs that paginate results
#[derive(Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct Page<T> {
    /// List of documents in the page
    pub docs: VecDeque<T>,
    /// Total number of documents
    #[serde(alias = "totalDocs")]
    pub total_docs: usize,
    /// Last page available
    #[serde(alias = "lastPage")]
    pub last_page: usize,
    /// Previous page number
    ///
    /// Note: Set to `None` if you are on the first page
    #[serde(alias = "prevPage")]
    pub prev_page: Option<usize>,
    /// Next page number
    ///
    /// Note: Set to `None` if you are on the last page
    #[serde(alias = "nextPage")]
    pub next_page: Option<usize>,
}
impl<T> PartialOrd for Page<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for Page<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.prev_page.is_none() && other.prev_page.is_none() {
            Ordering::Equal
        } else if self.prev_page.is_none() && other.prev_page.is_some() {
            Ordering::Less
        } else if other.prev_page.is_some() && other.prev_page.is_none() {
            Ordering::Greater
        } else {
            self.prev_page.unwrap().cmp(&other.prev_page.unwrap())
        }
    }
}
impl<T> PartialEq for Page<T> {
    fn eq(&self, other: &Self) -> bool {
        self.prev_page == other.prev_page
    }
}
impl<T> Eq for Page<T> { }

struct DateTimeVisitor;
impl DateTimeVisitor {
    fn from<T>(v: T) -> DateTime<Utc>
    where
        T: Into<i64>,
    {
        let ts: i64 = v.into();
        let nts = NaiveDateTime::from_timestamp(ts / 1000, ((ts % 1000) as u32) * 1_000_000);
        DateTime::from_utc(nts, Utc)
    }
}
impl<'a> de::Visitor<'a> for DateTimeVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a unix timestamp (including milliseconds)")
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::from(value as i64))
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::from(value))
    }
}
fn from_timestamp<'a, D>(d: D) -> Result<DateTime<Utc>, D::Error>
where
    D: de::Deserializer<'a>,
{
    d.deserialize_i64(DateTimeVisitor)
}

struct DurationVisitor;
impl DurationVisitor {
    fn from<T>(v: T) -> Duration
    where
        T: Into<u64>,
    {
        Duration::from_millis(v.into())
    }
}
impl<'a> de::Visitor<'a> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "an integer duration in milliseconds")
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::from(value))
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::from(value as u64))
    }
}
fn from_duration<'a, D>(d: D) -> Result<Duration, D::Error>
where
    D: de::Deserializer<'a>,
{
    d.deserialize_u64(DurationVisitor)
}

/// Structure used for deserializing rate limit errors
#[derive(Copy, Clone, Debug, Deserialize)]
pub struct BeatSaverRateLimit {
    /// DateTime when the rate limit will expire
    #[serde(deserialize_with = "from_timestamp")]
    pub reset: DateTime<Utc>,
    /// Duration of the rate limit
    #[serde(alias = "resetAfter", deserialize_with = "from_duration")]
    pub reset_after: Duration,
}
impl PartialEq for BeatSaverRateLimit {
    fn eq(&self, other: &Self) -> bool {
        self.reset == other.reset
    }
}
impl Eq for BeatSaverRateLimit {}
impl PartialOrd for BeatSaverRateLimit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for BeatSaverRateLimit {
    fn cmp(&self, other: &Self) -> Ordering {
        self.reset.cmp(&other.reset)
    }
}
impl Hash for BeatSaverRateLimit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.reset.hash(state);
    }
}
impl Display for BeatSaverRateLimit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Rate limited, expiring {}", self.reset)
    }
}

/// Converts the body of a 429 response to a BeatSaverApiError::RateLimitError
pub fn rate_limit<T: StdError>(data: Bytes) -> BeatSaverApiError<T> {
    let s = match String::from_utf8(data.as_ref().to_vec()) {
        Ok(s) => s,
        Err(e) => return e.into(),
    };
    let limit: BeatSaverRateLimit = match serde_json::from_str(s.as_str()) {
        Ok(b) => b,
        Err(e) => return e.into(),
    };
    BeatSaverApiError::RateLimitError(limit)
}

/// Error type for parsing a Map ID
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum MapIdError {
    /// Error returned when the provided hash is invalid
    ///
    /// This can occur in the following conditions:
    /// * The length of the hash is not 24
    /// * The hash contains non-hex characters
    #[error("specified hash is invalid")]
    InvalidHash,
    /// Error returned if the provided key is invalid
    ///
    /// This can occur in the following conditions:
    /// * Key is larger than a [usize][std::usize]
    /// * Key contains non-hex characters
    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),
}
impl From<FromHexError> for MapIdError {
    fn from(_: FromHexError) -> Self {
        Self::InvalidHash
    }
}

/// Specifier used to index a map
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MapId {
    /// Identifier is a map key (e.g. `1`)
    Key(usize),
    /// Identifier is a map hash (e.g. `fda568fc27c20d21f8dc6f3709b49b5cc96723be`)
    Hash(String),
}
impl Display for MapId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(key) => write!(f, "map key {}", key),
            Self::Hash(hash) => write!(f, "map hash {}", hash),
        }
    }
}
impl TryFrom<String> for MapId {
    type Error = MapIdError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.len() {
            40 => {
                hex::decode(&s)?;
                Ok(Self::Hash(s))
            }
            _ => Ok(Self::Key(usize::from_str_radix(s.as_str(), 16)?)),
        }
    }
}
impl TryFrom<&str> for MapId {
    type Error = MapIdError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.to_string().try_into()
    }
}
impl From<Map> for MapId {
    fn from(m: Map) -> Self {
        MapId::Hash(m.hash)
    }
}
impl From<&Map> for MapId {
    fn from(m: &Map) -> MapId {
        MapId::Hash(m.hash.clone())
    }
}

/// Error that could occur when querying the API
#[derive(Debug, Error)]
pub enum BeatSaverApiError<T: Debug + fmt::Display> {
    /// Error originated from the request backend
    #[error("{0}")]
    RequestError(T),
    /// Error originated from deserializing the api response
    #[error("{0}")]
    SerializeError(#[from] serde_json::Error),
    /// Argument provided is invalid
    #[error("Inalid argument: {0}")]
    ArgumentError(&'static str),
    /// Conversion to a [String][std::string::String] failed
    #[error("{0}")]
    Utf8Error(#[from] FromUtf8Error),
    /// Error in IO
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    /// Rate limit was hit while making the request
    #[error("API rate limit it (retry in {} ms)", .0.reset_after.as_millis())]
    RateLimitError(BeatSaverRateLimit),
}

#[cfg(all(feature = "async", not(feature = "sync")))]
pub use async_api::BeatSaverApiAsync as BeatSaverApi;
#[cfg(feature = "async")]
pub use async_api::BeatSaverApiAsync;

#[cfg(all(feature = "sync", not(feature = "async")))]
pub use sync_api::BeatSaverApiSync as BeatSaverApi;
#[cfg(feature = "sync")]
pub use sync_api::BeatSaverApiSync;

#[cfg(test)]
mod tests;
