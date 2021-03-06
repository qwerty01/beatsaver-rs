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
mod tests {
    use crate::map::Map;
    use crate::{BeatSaverApiError, Page};
    use bytes::Bytes;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fmt::{self, Display, Formatter};
    use url::Url;

    #[derive(Debug)]
    pub enum FakeError {}
    impl Display for FakeError {
        fn fmt(&self, _: &mut Formatter) -> fmt::Result {
            Ok(())
        }
    }
    impl Error for FakeError {}
    impl From<FakeError> for BeatSaverApiError<FakeError> {
        fn from(e: FakeError) -> Self {
            Self::RequestError(e)
        }
    }
    pub struct FakeClient {
        pub url: Url,
        pub data: Bytes,
    }
    impl FakeClient {
        pub fn new(url: Url, data: Bytes) -> Self {
            Self { url, data }
        }
    }
    pub struct FakeClientPaged {
        pub pages: HashMap<Url, Bytes>,
    }
    impl FakeClientPaged {
        pub fn new(pages: HashMap<Url, Bytes>) -> Self {
            Self { pages }
        }
    }

    #[test]
    fn test_page() {
        let data = r#"{"docs":[{"metadata":{"difficulties":{"easy":false,"normal":true,"hard":true,"expert":true,"expertPlus":true},"duration":0,"automapper":null,"characteristics":[{"name":"Standard","difficulties":{"easy":null,"normal":{"duration":417,"length":195,"bombs":4,"notes":301,"obstacles":24,"njs":10,"njsOffset":0},"hard":{"duration":417,"length":195,"bombs":4,"notes":486,"obstacles":24,"njs":10,"njsOffset":0},"expert":{"duration":417.5,"length":195,"bombs":4,"notes":620,"obstacles":24,"njs":10,"njsOffset":0},"expertPlus":{"duration":417.5,"length":195,"bombs":0,"notes":894,"obstacles":0,"njs":12,"njsOffset":0}}}],"songName":"Shut Up and Dance","songSubName":"WALK THE MOON","songAuthorName":"BennyDaBeast","levelAuthorName":"bennydabeast","bpm":128},"stats":{"downloads":418854,"plays":558,"downVotes":133,"upVotes":10763,"heat":395.8225333,"rating":0.9580848467461356},"description":"Difficulties: Expert+ (Added 11/15), Expert, Hard, Normal\r\nYouTube Preview: https://youtu.be/x9hJbTlPQUY","deletedAt":null,"_id":"5cff621148229f7d88fc77c9","key":"2144","name":"Shut Up and Dance - WALK THE MOON","uploader":{"_id":"5cff0b7298cc5a672c84e98d","username":"bennydabeast"},"uploaded":"2018-11-21T01:27:00.000Z","hash":"89cf8bb07afb3c59ae7b5ac00337d62261c36fb4","directDownload":"/cdn/2144/89cf8bb07afb3c59ae7b5ac00337d62261c36fb4.zip","downloadURL":"/api/download/key/2144","coverURL":"/cdn/2144/89cf8bb07afb3c59ae7b5ac00337d62261c36fb4.png"},{"metadata":{"difficulties":{"easy":false,"normal":true,"hard":true,"expert":true,"expertPlus":false},"duration":0,"automapper":null,"characteristics":[{"name":"Standard","difficulties":{"easy":null,"normal":{"duration":623.3125,"length":214,"bombs":0,"notes":462,"obstacles":25,"njs":10,"njsOffset":0},"hard":{"duration":623.3125,"length":214,"bombs":0,"notes":639,"obstacles":40,"njs":10,"njsOffset":0},"expert":{"duration":623.3125,"length":214,"bombs":0,"notes":825,"obstacles":40,"njs":10,"njsOffset":0},"expertPlus":null}}],"songName":"Mr. Blue Sky","songSubName":"Electric Light Orchestra","songAuthorName":"GreatYazer","levelAuthorName":"greatyazer","bpm":174},"stats":{"downloads":924827,"plays":39426,"downVotes":482,"upVotes":22614,"heat":94.0164429,"rating":0.9558554197954805},"description":"Channel your inner Baby Groot.  Normal, Hard, Expert\r\nSpecial thanks to BennydaBeast for his help on this track!","deletedAt":null,"_id":"5cff620d48229f7d88fc65f7","key":"570","name":"Mr. Blue Sky | Electric Light Orchestra","uploader":{"_id":"5cff0b7298cc5a672c84ea71","username":"greatyazer"},"uploaded":"2018-06-16T16:53:34.000Z","hash":"236173d5ba7dc379d480b9cb5fb6b4fa5abe77da","directDownload":"/cdn/570/236173d5ba7dc379d480b9cb5fb6b4fa5abe77da.zip","downloadURL":"/api/download/key/570","coverURL":"/cdn/570/236173d5ba7dc379d480b9cb5fb6b4fa5abe77da.jpg"},{"metadata":{"difficulties":{"easy":false,"normal":false,"hard":false,"expert":true,"expertPlus":false},"duration":0,"automapper":null,"characteristics":[{"name":"Standard","difficulties":{"easy":null,"normal":null,"hard":null,"expert":{"duration":476.7637634277344,"length":173,"bombs":52,"notes":722,"obstacles":28,"njs":14,"njsOffset":0},"expertPlus":null}}],"songName":"Caramelldansen (Speedcake Remix)","songSubName":"Caramell","songAuthorName":"Dack","levelAuthorName":"Dack","bpm":165},"stats":{"downloads":255909,"plays":0,"downVotes":247,"upVotes":13125,"heat":604.9830484,"rating":0.953954336380672},"description":"Preview: https://youtu.be/V5p0HOzunY0\n\n\nPatreon: https://www.patreon.com/Dack","deletedAt":null,"_id":"5cff621548229f7d88fc8904","key":"3cf5","name":"Caramelldansen","uploader":{"_id":"5cff0b7598cc5a672c852c6f","username":"dack"},"uploaded":"2019-03-09T22:54:54.000Z","hash":"cf5e32d6b7f30095f7198da5894139c92336cad7","directDownload":"/cdn/3cf5/cf5e32d6b7f30095f7198da5894139c92336cad7.zip","downloadURL":"/api/download/key/3cf5","coverURL":"/cdn/3cf5/cf5e32d6b7f30095f7198da5894139c92336cad7.jpg"},{"metadata":{"difficulties":{"easy":false,"normal":false,"hard":true,"expert":true,"expertPlus":false},"duration":0,"automapper":null,"characteristics":[{"name":"Standard","difficulties":{"easy":null,"normal":null,"hard":{"duration":475,"length":226,"bombs":0,"notes":620,"obstacles":10,"njs":10,"njsOffset":0},"expert":{"duration":475,"length":226,"bombs":0,"notes":738,"obstacles":11,"njs":12,"njsOffset":0},"expertPlus":null}}],"songName":"Feel Invincible","songSubName":"Skillet","songAuthorName":"Rustic","levelAuthorName":"rustic","bpm":126},"stats":{"downloads":264718,"plays":3479,"downVotes":90,"upVotes":7118,"heat":231.7496095,"rating":0.9538897573416698},"description":"Expert / Hard\r\nhttps://www.youtube.com/watch?v=nq-Qul4XxbE","deletedAt":null,"_id":"5cff620f48229f7d88fc6eba","key":"121f","name":"Skillet - Feel Invincible","uploader":{"_id":"5cff0b7298cc5a672c84e8c4","username":"rustic"},"uploaded":"2018-08-27T16:47:05.000Z","hash":"2e9ab6e1fb8055649e241cade98b018926cc93a8","directDownload":"/cdn/121f/2e9ab6e1fb8055649e241cade98b018926cc93a8.zip","downloadURL":"/api/download/key/121f","coverURL":"/cdn/121f/2e9ab6e1fb8055649e241cade98b018926cc93a8.jpg"},{"metadata":{"difficulties":{"easy":true,"normal":true,"hard":true,"expert":true,"expertPlus":false},"duration":0,"automapper":null,"characteristics":[{"name":"Standard","difficulties":{"easy":{"duration":418,"length":200,"bombs":0,"notes":216,"obstacles":0,"njs":10,"njsOffset":0},"normal":{"duration":418,"length":200,"bombs":0,"notes":388,"obstacles":8,"njs":10,"njsOffset":0},"hard":{"duration":418,"length":200,"bombs":0,"notes":514,"obstacles":10,"njs":11,"njsOffset":0},"expert":{"duration":418,"length":200,"bombs":0,"notes":560,"obstacles":12,"njs":12,"njsOffset":0},"expertPlus":null}}],"songName":"Believer (100k ver.)","songSubName":"Imagine Dragons","songAuthorName":"Rustic","levelAuthorName":"rustic","bpm":125},"stats":{"downloads":511866,"plays":9381,"downVotes":215,"upVotes":11898,"heat":379.8995099,"rating":0.9538005825373931},"description":"This is one of the 22 maps that were mapped for the 100k Contest where you can win over $7,000 in prizes. Go to https://bsaber.com/100k-contest/ to register!","deletedAt":null,"_id":"5cff621148229f7d88fc76ec","key":"1fef","name":"Imagine Dragons - Believer (100k ver.) | 100k Contest","uploader":{"_id":"5cff0b7298cc5a672c84e8c4","username":"rustic"},"uploaded":"2018-11-12T17:53:58.000Z","hash":"9a7a5beadfdd1c7c0f137ecba6e5f6ff377eb390","directDownload":"/cdn/1fef/9a7a5beadfdd1c7c0f137ecba6e5f6ff377eb390.zip","downloadURL":"/api/download/key/1fef","coverURL":"/cdn/1fef/9a7a5beadfdd1c7c0f137ecba6e5f6ff377eb390.jpg"},{"metadata":{"difficulties":{"easy":false,"normal":false,"hard":true,"expert":true,"expertPlus":false},"duration":0,"automapper":null,"characteristics":[{"name":"Standard","difficulties":{"easy":null,"normal":null,"hard":{"duration":501.19000244140625,"length":227,"bombs":0,"notes":671,"obstacles":16,"njs":10,"njsOffset":0},"expert":{"duration":501.19000244140625,"length":227,"bombs":0,"notes":831,"obstacles":16,"njs":10,"njsOffset":0},"expertPlus":null}}],"songName":"Daddy","songSubName":"PSY","songAuthorName":"Fafurion","levelAuthorName":"fafurion","bpm":132},"stats":{"downloads":249499,"plays":2330,"downVotes":122,"upVotes":8354,"heat":243.8938387,"rating":0.9537082538199915},"description":"Insanely fun dance map! Enjoy!\r\nSee the map in action (Expert): https://www.youtube.com/watch?v=cWz6flYGs20\r\n\r\nThank you to my playtesters:\r\nQTPop (https://www.twitch.tv/qtpop)\r\nDuoVR (https://www.twitch.tv/duovr)\r\nSourgurl (https://www.twitch.tv/sourgurl)\r\nRexxxzi (https://www.twitch.tv/rexxxzi)\r\nAshleyriott (https://www.twitch.tv/ashleyriott)\r\n\r\nDiscord: @Fufu#5452","deletedAt":null,"_id":"5cff620f48229f7d88fc6f6c","key":"133b","name":"Daddy - PSY","uploader":{"_id":"5cff0b7398cc5a672c84f945","username":"fafurion"},"uploaded":"2018-09-02T23:43:45.000Z","hash":"dc489921185f92dfecb9cb07b84fc556123bd134","directDownload":"/cdn/133b/dc489921185f92dfecb9cb07b84fc556123bd134.zip","downloadURL":"/api/download/key/133b","coverURL":"/cdn/133b/dc489921185f92dfecb9cb07b84fc556123bd134.jpg"},{"metadata":{"difficulties":{"easy":false,"normal":false,"hard":true,"expert":true,"expertPlus":false},"duration":0,"automapper":null,"characteristics":[{"name":"Standard","difficulties":{"easy":null,"normal":null,"hard":{"duration":523.625,"length":245,"bombs":0,"notes":633,"obstacles":13,"njs":10,"njsOffset":0},"expert":{"duration":523.5650024414062,"length":245,"bombs":0,"notes":880,"obstacles":19,"njs":12,"njsOffset":0},"expertPlus":null}}],"songName":"Uprising","songSubName":"Muse","songAuthorName":"Rustic","levelAuthorName":"rustic","bpm":128},"stats":{"downloads":455551,"plays":11598,"downVotes":163,"upVotes":9190,"heat":84.4191212,"rating":0.9517991988617799},"description":"Expert / Hard\r\nhttps://youtu.be/WpFUt3UNp7c","deletedAt":null,"_id":"5cff620d48229f7d88fc655e","key":"4c6","name":"Muse - Uprising","uploader":{"_id":"5cff0b7298cc5a672c84e8c4","username":"rustic"},"uploaded":"2018-06-11T21:47:41.000Z","hash":"00e5671e594a6fe621c3605fcc5a0e4466ba6478","directDownload":"/cdn/4c6/00e5671e594a6fe621c3605fcc5a0e4466ba6478.zip","downloadURL":"/api/download/key/4c6","coverURL":"/cdn/4c6/00e5671e594a6fe621c3605fcc5a0e4466ba6478.jpg"},{"metadata":{"difficulties":{"easy":false,"expert":true,"expertPlus":true,"hard":false,"normal":false},"duration":0,"automapper":null,"characteristics":[{"difficulties":{"easy":null,"expert":{"duration":547,"length":234,"njs":12,"njsOffset":0,"bombs":0,"notes":705,"obstacles":10},"expertPlus":{"duration":547,"length":234,"njs":16,"njsOffset":0,"bombs":0,"notes":876,"obstacles":10},"hard":null,"normal":null},"name":"Standard"}],"levelAuthorName":"KikaeAeon","songAuthorName":"League of Legends & Against The Current","songName":"Legends Never Die","songSubName":"","bpm":140},"stats":{"downloads":141233,"plays":0,"downVotes":86,"upVotes":6203,"heat":997.2528853,"rating":0.9513775893989509},"description":"A special request from Prima1URGE","deletedAt":null,"_id":"5d91d6c1871b1a0006f9b3e7","key":"66e6","name":"League of Legends - Legends never die (ft. Against The Current)","uploader":{"_id":"5cff0b7498cc5a672c85109b","username":"kikaeaeon"},"hash":"732bd4072b89d4b3bf0e63db812a7ffc3096e837","uploaded":"2019-09-30T10:19:45.606Z","directDownload":"/cdn/66e6/732bd4072b89d4b3bf0e63db812a7ffc3096e837.zip","downloadURL":"/api/download/key/66e6","coverURL":"/cdn/66e6/732bd4072b89d4b3bf0e63db812a7ffc3096e837.jpg"},{"metadata":{"difficulties":{"easy":false,"normal":false,"hard":true,"expert":true,"expertPlus":false},"duration":0,"automapper":null,"characteristics":[{"name":"Standard","difficulties":{"easy":null,"normal":null,"hard":{"duration":578,"length":194,"bombs":8,"notes":519,"obstacles":86,"njs":13,"njsOffset":0},"expert":{"duration":578,"length":194,"bombs":10,"notes":679,"obstacles":86,"njs":16,"njsOffset":0},"expertPlus":null}}],"songName":"Flamingo","songSubName":"Kero Kero Bonito","songAuthorName":"ETAN","levelAuthorName":"ETAN","bpm":178},"stats":{"downloads":141034,"plays":0,"downVotes":89,"upVotes":6307,"heat":744.9760285,"rating":0.9513314992226289},"description":"edited: NJS was a tad slow my b\r\nfuramingo \r\noh oh ooh oh\r\nthis needed to be remapped\r\n \r\nPlease check out Kero Kero Bonito and the rest of their songs\r\nI'll be mappin more of em soon\r\n\r\nBPM 178\r\nFlamingo (Expert)\r\nShrimp (Hard)\r\n\r\nhave fun \r\n\r\ngimme feedback on Discord\r\nETAN#8341","deletedAt":null,"_id":"5cff621748229f7d88fc93fc","key":"4e6f","name":"Kero Kero Bonito - Flamingo","uploader":{"_id":"5cff0b7798cc5a672c855775","username":"etan"},"uploaded":"2019-05-22T00:46:47.000Z","hash":"585ee25e654ebf5db5aa0ec02c3bcecbaccf3e0b","directDownload":"/cdn/4e6f/585ee25e654ebf5db5aa0ec02c3bcecbaccf3e0b.zip","downloadURL":"/api/download/key/4e6f","coverURL":"/cdn/4e6f/585ee25e654ebf5db5aa0ec02c3bcecbaccf3e0b.jpg"},{"metadata":{"difficulties":{"easy":false,"normal":false,"hard":false,"expert":true,"expertPlus":false},"duration":0,"automapper":null,"characteristics":[{"name":"Standard","difficulties":{"easy":null,"normal":null,"hard":null,"expert":{"duration":689.8004760742188,"length":242,"bombs":16,"notes":1093,"obstacles":16,"njs":14,"njsOffset":0},"expertPlus":null}}],"songName":"IGNITE","songSubName":"Aoi Eir","songAuthorName":"Joetastic","levelAuthorName":"Joetastic","bpm":171},"stats":{"downloads":264817,"plays":0,"downVotes":141,"upVotes":7891,"heat":451.3583461,"rating":0.9502372997935349},"description":"Trying to get this version ranked! New version with changes according to the ranking criteria: Double directional notes fixed, flow improvements, removed fast dodge walls on bridge section.","deletedAt":null,"_id":"5cff621248229f7d88fc7b21","key":"26f6","name":"IGNITE (Ranked Version) [Sword Art Online Season 2 Opening] - Aoi Eir","uploader":{"_id":"5cff0b7498cc5a672c85050e","username":"joetastic"},"uploaded":"2018-12-20T01:21:47.000Z","hash":"125b07ebcc06fe9667e83fc2d6b9ae5ecbc72e8c","directDownload":"/cdn/26f6/125b07ebcc06fe9667e83fc2d6b9ae5ecbc72e8c.zip","downloadURL":"/api/download/key/26f6","coverURL":"/cdn/26f6/125b07ebcc06fe9667e83fc2d6b9ae5ecbc72e8c.jpg"}],"totalDocs":35367,"lastPage":3536,"prevPage":null,"nextPage":1}"#;

        let page: Page<Map> = serde_json::from_str(data).unwrap();

        assert_eq!(page.docs.len(), 10);
        assert_eq!(page.total_docs, 35367);
        assert_eq!(page.last_page, 3536);
        assert_eq!(page.prev_page, None);
        assert_eq!(page.next_page, Some(1));
    }
}
