//! Bangumi v0 API 数据模型。
//!
//! 长期字段全集以 M4-1 数据可行性核对表为准（docs/AniTrail-Slint实现TODO.md M4-1）；
//! 当前收录基础接口已确认的关键字段，`deny_unknown_fields` 不使用——
//! 对端加字段不应破坏本地解析。

use serde::Deserialize;

/// `GET /v0/me`。
#[derive(Debug, Clone, Deserialize)]
pub struct Me {
    pub id: u32,
    pub username: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub avatar: Option<MemberAvatar>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemberAvatar {
    #[serde(default, rename = "small")]
    pub small: Option<String>,
    #[serde(default, rename = "large")]
    pub large: Option<String>,
}

/// `GET /v0/subjects/{subject_id}`。
#[derive(Debug, Clone, Deserialize)]
pub struct Subject {
    pub id: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_cn: String,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub images: Option<SubjectImages>,
    #[serde(default)]
    pub rating: Option<SubjectRating>,
    #[serde(default)]
    pub collection: Option<SubjectCollectionCounts>,
    #[serde(default)]
    pub eps: Option<u32>,
    #[serde(default)]
    pub total_episodes: Option<u32>,
    #[serde(default)]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubjectImages {
    #[serde(default, rename = "large")]
    pub large: Option<String>,
    #[serde(default, rename = "common")]
    pub common: Option<String>,
    #[serde(default, rename = "medium")]
    pub medium: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubjectRating {
    #[serde(default)]
    pub rank: Option<u32>,
    #[serde(default)]
    pub total: Option<u32>,
    #[serde(default)]
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubjectCollectionCounts {
    #[serde(default)]
    pub wish: u32,
    #[serde(default)]
    pub collect: u32,
    #[serde(default)]
    pub doing: u32,
    #[serde(default)]
    pub on_hold: u32,
    #[serde(default)]
    pub dropped: u32,
}
