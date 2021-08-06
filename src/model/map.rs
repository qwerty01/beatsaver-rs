use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use url::Url;

use super::user::UserDetail;

#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct MapDetail {
    pub automapper: bool,
    pub curator: String,
    pub description: String,
    pub id: String,
    pub metadata: MapDetailMetadata,
    pub name: String,
    pub qualified: bool,
    pub ranked: bool,
    pub stats: MapStats,
    pub uploaded: DateTime<Utc>,
    pub uploader: UserDetail,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct MapDetailMetadata {
    pub bpm: f32,
    pub duration: u32,
    pub level_author: String,
    pub song_author: String,
    pub song_name: String,
    pub song_sub_name: String,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct MapStats {
    pub downloads: u32,
    pub downvotes: u32,
    pub plays: u32,
    pub score: f32,
    pub upvotes: u32,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct MapVersion {
    pub cover_url: Url,
    pub created_at: DateTime<Utc>,
    pub diffs: Vec<MapDifficulty>,
    pub download_url: Url,
    pub feedback: String,
    pub hash: String,
    pub key: String,
    pub preview_url: Url,
    pub sage_score: u16,
    pub state: MapState,
    pub test_play_at: DateTime<Utc>,
    pub testplays: Vec<MapTestPlay>,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct MapDifficulty {
    pub bombs: u32,
    pub characteristic: MapCharacteristic,
    pub chroma: bool,
    pub cinema: bool,
    pub difficulty: MapDifficultyLevel,
    pub events: u32,
    pub length: f64,
    pub me: bool,
    pub ne: bool,
    pub njs: f32,
    pub notes: u32,
    pub nps: f64,
    pub obstacles: u32,
    pub offset: f32,
    pub parity_summary: MapParitySummary,
    pub seconds: f64,
    pub stars: f32,
}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, Serialize, Deserialize,
)]
pub enum MapState {
    Upload,
    TestPlay,
    Published,
    Feedback,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct MapTestPlay {
    pub created_at: DateTime<Utc>,
    pub feedback: String,
    pub feedback_at: DateTime<Utc>,
    pub user: UserDetail,
    pub video: String,
}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, Serialize, Deserialize,
)]
pub enum MapCharacteristic {
    Standard,
    #[display(fmt = "One Saber")]
    OneSaber,
    #[display(fmt = "No Arrows")]
    NoArrows,
    #[display(fmt = "90 Degrees")]
    NinetyDegree,
    #[display(fmt = "360 Degrees")]
    ThreeSixtyDegree,
    #[display(fmt = "Light Show")]
    LightShow,
    Lawless,
}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, Serialize, Deserialize,
)]
pub enum MapDifficultyLevel {
    Easy,
    Normal,
    Hard,
    Expert,
    #[display(fmt = "Expert+")]
    ExpertPlus,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct MapParitySummary {
    pub errors: u32,
    pub resets: u32,
    pub warns: u32,
}
