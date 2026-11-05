//! AniTrail 核心库： Bangumi 对接层复导出、Slint 画面桥；后续接入 db/sync/domain。

pub mod media;

#[cfg(test)]
mod tests;

pub use ani_trail_bangumi as bangumi;
