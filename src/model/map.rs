use chrono::{DateTime, Utc};

use super::user::UserDetail;

pub struct MapDetail {
    automapper: bool,
    curator: String,
    description: String,
    id: String,
    metadata: MapDetailMetadata,
    name: String,
    qualified: bool,
    ranked: bool,
    stats: MapStats,
    uploaded: DateTime<Utc>,
    uploader: UserDetail,
}
