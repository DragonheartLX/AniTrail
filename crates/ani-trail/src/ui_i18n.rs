//! 文案常量（D17 双源同键：docs/规划/09-文案表.md 的键名 ↔ 本模块常量，改任一处必须同步另一处）。

/// 09 §4 · 开屏页
pub mod splash {
    /// splash.slogan
    pub const SLOGAN: &str = "记录每一部动画的轨迹";
    /// splash.loading-bangumi
    pub const LOADING_BANGUMI: &str = "正在同步 Bangumi 数据…";
    /// splash.version — 实际取应用版本号，telemetry 注记 TDesign / tela-ui 基座
    pub const VERSION_LABEL: &str = "TDesign / tela-ui";
}
