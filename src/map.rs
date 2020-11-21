use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::BeatSaverUser;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapDifficulties {
    pub easy: bool,
    pub normal: bool,
    pub hard: bool,
    pub expert: bool,
    #[serde(alias = "expertPlus")]
    pub expert_plus: bool
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapDifficltyCharacteristic {
    pub duration: f32,
    pub length: usize,
    pub njs: usize,
    #[serde(alias = "njsOffset")]
    pub njs_offset: i64,
    pub bombs: usize,
    pub notes: usize,
    pub obstacles: usize
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapDifficultyCharacteristics {
    pub easy: Option<MapDifficltyCharacteristic>,
    pub normal: Option<MapDifficltyCharacteristic>,
    pub hard: Option<MapDifficltyCharacteristic>,
    pub expert: Option<MapDifficltyCharacteristic>,
    #[serde(alias = "expertPlus")]
    pub expert_plus: Option<MapDifficltyCharacteristic>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapCharacteristics {
    pub difficulties: MapDifficultyCharacteristics,
    pub name: String
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapMetadata {
    pub difficulties: MapDifficulties,
    pub duration: usize,
    pub automapper: Option<String>,
    pub characteristics: Vec<MapCharacteristics>,
    #[serde(alias = "levelAuthorName")]
    pub level_author: String,
    #[serde(alias = "songAuthorName")]
    pub song_author: String,
    #[serde(alias = "songName")]
    pub song_name: String,
    #[serde(alias = "songSubName")]
    pub song_sub_name: String,
    pub bpm: usize
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapStats {
    pub downloads: usize,
    pub plays: usize,
    #[serde(alias = "downVotes")]
    pub downvotes: usize,
    #[serde(alias = "upVotes")]
    pub upvotes: usize,
    pub heat: f32,
    pub rating: f32
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Map {
    pub metadata: MapMetadata,
    pub stats: MapStats,
    pub description: String,
    #[serde(alias = "_id")]
    pub id: String,
    pub key: String,
    pub name: String,
    pub uploader: BeatSaverUser,
    pub hash: String,
    pub uploaded: DateTime<Utc>,
    #[serde(alias = "directDownload")]
    pub direct_download: String,
    #[serde(alias = "downloadURL")]
    pub download: String,
    #[serde(alias = "coverURL")]
    pub cover: String
}

#[cfg(test)]
mod tests {
    use serde_json;
    use chrono::DateTime;
    use crate::map::Map;
    
    #[test]
    fn test_map() {
        let data = r#"
        {
            "metadata": {
                "difficulties": {
                    "easy": false,
                    "normal": true,
                    "hard": true,
                    "expert":true,
                    "expertPlus":true
                },
                "duration": 0,
                "automapper": null,
                "characteristics": [{
                    "name":"Standard",
                    "difficulties": {
                        "easy": null,
                        "normal": {
                            "duration": 417,
                            "length": 195,
                            "bombs": 4,
                            "notes": 301,
                            "obstacles": 24,
                            "njs": 10,
                            "njsOffset": 0
                        },
                        "hard": {
                            "duration": 417,
                            "length": 195,
                            "bombs": 4,
                            "notes": 486,
                            "obstacles": 24,
                            "njs": 10,
                            "njsOffset": 0
                        },
                        "expert": {
                            "duration": 417.5,
                            "length": 195,
                            "bombs": 4,
                            "notes": 620,
                            "obstacles": 24,
                            "njs": 10,
                            "njsOffset": 0
                        },
                        "expertPlus": {
                            "duration": 417.5,
                            "length": 195,
                            "bombs": 0,
                            "notes": 894,
                            "obstacles": 0,
                            "njs": 12,
                            "njsOffset": 0
                        }
                    }
                }],
                "songName": "Shut Up and Dance",
                "songSubName": "WALK THE MOON",
                "songAuthorName": "BennyDaBeast",
                "levelAuthorName": "bennydabeast",
                "bpm":128
            },
            "stats": {
                "downloads": 418854,
                "plays": 558,
                "downVotes": 133,
                "upVotes": 10763,
                "heat": 395.8225333,
                "rating": 0.9580848467461356
            },
            "description": "Difficulties: Expert+ (Added 11/15), Expert, Hard, Normal\r\nYouTube Preview: https://youtu.be/x9hJbTlPQUY",
            "deletedAt": null,
            "_id": "5cff621148229f7d88fc77c9",
            "key": "2144",
            "name": "Shut Up and Dance - WALK THE MOON",
            "uploader": {
                "_id": "5cff0b7298cc5a672c84e98d",
                "username": "bennydabeast"
            },
            "uploaded": "2018-11-21T01:27:00.000Z",
            "hash": "89cf8bb07afb3c59ae7b5ac00337d62261c36fb4",
            "directDownload": "/cdn/2144/89cf8bb07afb3c59ae7b5ac00337d62261c36fb4.zip",
            "downloadURL": "/api/download/key/2144",
            "coverURL": "/cdn/2144/89cf8bb07afb3c59ae7b5ac00337d62261c36fb4.png"
        }"#;

        let v: Map = serde_json::from_str(data).unwrap();

        let difficulties = v.metadata.difficulties;
        assert_eq!(difficulties.easy, false);
        assert_eq!(difficulties.normal, true);
        assert_eq!(difficulties.hard, true);
        assert_eq!(difficulties.expert, true);
        assert_eq!(difficulties.expert_plus, true);

        assert_eq!(v.metadata.duration, 0);
        assert_eq!(v.metadata.automapper, None);

        assert_eq!(v.metadata.characteristics.len(), 1);
        let characteristics = &v.metadata.characteristics[0];
        assert_eq!(characteristics.name, "Standard");

        assert_eq!(characteristics.difficulties.easy, None);

        assert!(characteristics.difficulties.normal.is_some());
        let normal = characteristics.difficulties.normal.as_ref().unwrap();
        assert_eq!(normal.duration, 417f32);
        assert_eq!(normal.length, 195);
        assert_eq!(normal.bombs, 4);
        assert_eq!(normal.notes, 301);
        assert_eq!(normal.obstacles, 24);
        assert_eq!(normal.njs, 10);
        assert_eq!(normal.njs_offset, 0);

        assert!(characteristics.difficulties.hard.is_some());
        let hard = characteristics.difficulties.hard.as_ref().unwrap();
        assert_eq!(hard.duration, 417f32);
        assert_eq!(hard.length, 195);
        assert_eq!(hard.bombs, 4);
        assert_eq!(hard.notes, 486);
        assert_eq!(hard.obstacles, 24);
        assert_eq!(hard.njs, 10);
        assert_eq!(hard.njs_offset, 0);

        assert!(characteristics.difficulties.expert.is_some());
        let expert = characteristics.difficulties.expert.as_ref().unwrap();
        assert_eq!(expert.duration, 417.5f32);
        assert_eq!(expert.length, 195);
        assert_eq!(expert.bombs, 4);
        assert_eq!(expert.notes, 620);
        assert_eq!(expert.obstacles, 24);
        assert_eq!(expert.njs, 10);
        assert_eq!(expert.njs_offset, 0);

        assert!(characteristics.difficulties.expert_plus.is_some());
        let expert_plus = characteristics.difficulties.expert_plus.as_ref().unwrap();
        assert_eq!(expert_plus.duration, 417.5f32);
        assert_eq!(expert_plus.length, 195);
        assert_eq!(expert_plus.bombs, 0);
        assert_eq!(expert_plus.notes, 894);
        assert_eq!(expert_plus.obstacles, 0);
        assert_eq!(expert_plus.njs, 12);
        assert_eq!(expert_plus.njs_offset, 0);

        assert_eq!(v.metadata.song_name, "Shut Up and Dance");
        assert_eq!(v.metadata.song_sub_name, "WALK THE MOON");
        assert_eq!(v.metadata.song_author, "BennyDaBeast");
        assert_eq!(v.metadata.level_author, "bennydabeast");
        assert_eq!(v.metadata.bpm, 128);

        assert_eq!(v.stats.downloads, 418854);
        assert_eq!(v.stats.plays, 558);
        assert_eq!(v.stats.downvotes, 133);
        assert_eq!(v.stats.upvotes, 10763);
        assert_eq!(v.stats.heat, 395.8225333f32);
        assert_eq!(v.stats.rating, 0.9580848467461356f32);

        assert_eq!(v.description, "Difficulties: Expert+ (Added 11/15), Expert, Hard, Normal\r\nYouTube Preview: https://youtu.be/x9hJbTlPQUY");
        assert_eq!(v.key, "2144");
        assert_eq!(v.name, "Shut Up and Dance - WALK THE MOON");
        assert_eq!(v.uploader.id, "5cff0b7298cc5a672c84e98d");
        assert_eq!(v.uploader.username, "bennydabeast");
        assert_eq!(v.uploaded, DateTime::parse_from_rfc3339("2018-11-21T01:27:00.000Z").unwrap());
        assert_eq!(v.hash, "89cf8bb07afb3c59ae7b5ac00337d62261c36fb4");
        assert_eq!(v.direct_download, "/cdn/2144/89cf8bb07afb3c59ae7b5ac00337d62261c36fb4.zip");
        assert_eq!(v.download, "/api/download/key/2144");
        assert_eq!(v.cover, "/cdn/2144/89cf8bb07afb3c59ae7b5ac00337d62261c36fb4.png");
    }
}
