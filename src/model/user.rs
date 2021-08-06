use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

/// User information
///
/// This information is pulled from https://api.beatsaver.com/users/id/{user_id}
#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct UserDetail {
    /// User ID
    pub id: u32,
    /// User name
    pub name: String,
    /// User hash
    pub hash: String,
    /// Avatar URL
    pub avatar: Url,
    /// User statistics
    pub stats: UserStats,
}

/// User statistics
///
/// Normally used with a [`UserDetail`]
#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct UserStats {
    /// Total upvotes on user maps
    #[serde(rename = "totalUpvotes")]
    pub total_upvotes: u32,
    /// Total downvotes on user's maps
    #[serde(rename = "totalDownvotes")]
    pub total_downvotes: u32,
    /// Total maps created
    #[serde(rename = "totalMaps")]
    pub total_maps: u32,
    /// Number of ranked maps
    #[serde(rename = "rankedMaps")]
    pub ranked_maps: u32,
    /// Average BPM of maps created
    #[serde(rename = "avgBpm")]
    pub average_bpm: f32,
    /// Average score of maps created
    #[serde(rename = "avgScore")]
    pub average_score: f32,
    /// Average duration of maps created
    #[serde(rename = "avgDuration")]
    pub average_duration: f32,
    /// First upload date
    #[serde(rename = "firstUpload")]
    pub first_upload: DateTime<Utc>,
    /// Most recent upload date
    #[serde(rename = "lastUpload")]
    pub last_upload: DateTime<Utc>,
    /// Statistics of difficulties created
    #[serde(rename = "diffStats")]
    pub diff_stats: UserDiffStats,
}

/// Statistics of maps created on each difficulty
///
/// Normally used with a [`UserStats`]
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize,
)]
pub struct UserDiffStats {
    /// Total number of difficulties created
    ///
    /// Sum of the other fields
    pub total: u32,
    /// Number of easy maps created
    pub easy: u32,
    /// Number of normal maps created
    pub normal: u32,
    /// Number of hard maps created
    pub hard: u32,
    /// Number of expert maps created
    pub expert: u32,
    /// Number of expert+ maps created
    #[serde(rename = "expertPlus")]
    pub expert_plus: u32,
}
