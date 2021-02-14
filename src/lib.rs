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
use hex::{self, FromHexError};
use lazy_static::lazy_static;
use map::Map;
use serde::{de, Deserialize, Serialize};
use serde_json;
use std::collections::VecDeque;
use std::convert::{From, TryFrom, TryInto};
use std::error::Error;
use std::fmt;
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeatSaverUser {
    /// User ID (e.g. `5fbe7cd60192c700062b2a1f`)
    #[serde(alias = "_id")]
    pub id: String,
    /// User name (e.g. `qwerty01`)
    pub username: String,
}

/// Page metadata for APIs that paginate results
#[derive(Clone, Serialize, Deserialize)]
pub struct Page<T: Serialize> {
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
#[derive(Clone, Debug, Deserialize)]
pub struct BeatSaverRateLimit {
    /// DateTime when the rate limit will expire
    #[serde(deserialize_with = "from_timestamp")]
    pub reset: DateTime<Utc>,
    /// Duration of the rate limit
    #[serde(alias = "resetAfter", deserialize_with = "from_duration")]
    pub reset_after: Duration,
}

/// Converts the body of a 429 response to a BeatSaverApiError::RateLimitError
pub fn rate_limit<T: Error>(data: Bytes) -> BeatSaverApiError<T> {
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
#[derive(Debug, Clone, PartialEq)]
pub enum MapIdError {
    /// Error returned when the provided hash is invalid
    ///
    /// This can occur in the following conditions:
    /// * The length of the hash is not 24
    /// * The hash contains non-hex characters
    InvalidHash,
    /// Error returned if the provided key is invalid
    ///
    /// This can occur in the following conditions:
    /// * Key is larger than a [usize][std::usize]
    /// * Key contains non-hex characters
    ParseIntError(ParseIntError),
}
impl fmt::Display for MapIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidHash => write!(f, "Specified hash is invalid"),
            Self::ParseIntError(e) => e.fmt(f),
        }
    }
}
impl Error for MapIdError {}
impl From<ParseIntError> for MapIdError {
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}
impl From<FromHexError> for MapIdError {
    fn from(_: FromHexError) -> Self {
        Self::InvalidHash
    }
}

/// Specifier used to index a map
#[derive(Debug, Clone, PartialEq)]
pub enum MapId {
    /// Identifier is a map key (e.g. `1`)
    Key(usize),
    /// Identifier is a map hash (e.g. `fda568fc27c20d21f8dc6f3709b49b5cc96723be`)
    Hash(String),
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
impl Into<MapId> for Map {
    fn into(self) -> MapId {
        MapId::Hash(self.hash)
    }
}
impl Into<MapId> for &Map {
    fn into(self) -> MapId {
        MapId::Hash(self.hash.clone())
    }
}

/// Error that could occur when querying the API
#[derive(Debug)]
pub enum BeatSaverApiError<T: fmt::Display> {
    /// Error originated from the request backend
    RequestError(T),
    /// Error originated from deserializing the api response
    SerializeError(serde_json::Error),
    /// Argument provided is invalid
    ArgumentError(&'static str),
    /// Conversion to a [String][std::string::String] failed
    Utf8Error(FromUtf8Error),
    /// Error in IO
    IoError(std::io::Error),
    /// Rate limit was hit while making the request
    RateLimitError(BeatSaverRateLimit),
}
impl<T: fmt::Display> fmt::Display for BeatSaverApiError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::RequestError(e) => <T as fmt::Display>::fmt(e, f),
            Self::SerializeError(e) => e.fmt(f),
            Self::ArgumentError(a) => write!(f, "Invalid argument: {}", a),
            Self::Utf8Error(e) => e.fmt(f),
            Self::IoError(e) => e.fmt(f),
            Self::RateLimitError(e) => {
                write!(
                    f,
                    "API rate limit hit (retry in {} ms)",
                    e.reset_after.as_millis()
                )
            }
        }
    }
}
impl<T: fmt::Display> From<serde_json::Error> for BeatSaverApiError<T> {
    fn from(e: serde_json::Error) -> Self {
        Self::SerializeError(e)
    }
}
impl<T: fmt::Display> From<FromUtf8Error> for BeatSaverApiError<T> {
    fn from(e: FromUtf8Error) -> Self {
        Self::Utf8Error(e)
    }
}
impl<T: fmt::Display> From<std::io::Error> for BeatSaverApiError<T> {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
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
