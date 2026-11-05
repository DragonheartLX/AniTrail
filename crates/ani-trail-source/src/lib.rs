//! 番迹 (AniTrail) 播放数据源契约层（D33）。
//!
//! 职责：**Provider 插槽 + 形状冻结**，不含任何抓取/解析实现（那在 M6-6 起按方言实现落此 crate）：
//! * [`provider::SourceProvider`] —— 数据源统一接口（kazumi 规则 Provider 为第一个 impl；BT Provider 为 P1/M10 候选，D32）；
//! * [`SearchEntry`] / [`EpisodeSource`] —— 选集/直链统一形状，引擎只见这些类型；
//! * [`SelectorDialect`] —— 解析方言声明（D31：XPath + CSS 双派系都收）。
//!
//! 禁区：本 crate 不依赖 slint、不依赖 mpv/libtorrent/xpath 具体引擎——
//! 那些 required deps 属于各 Provider 实现与 `ani-trail-player` 引擎层，须维持"重依赖不进契约"（拆分标准 ①）。

pub mod error;
pub mod provider;

#[cfg(test)]
mod tests;

pub use error::Error;
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// 搜索条目（Provider 搜索结果）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchEntry {
    pub title: String,
    /// Provider 自持回跳凭据（规则字段、站点 id、磁力 hash…）——对上层不透明。
    pub ref_id: String,
}

/// 一季/一部作品的剧集源：Provider 给出按序号的直链与标题。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpisodeSource {
    /// 站内序号（1 起）；与 Bangumi episode_id 的映射由应用层（core 同步域）人工确认表建立。
    pub index: u32,
    pub title: Option<String>,
    pub uri: String,
}

/// 解析方言（D31：双派系都收）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectorDialect {
    /// Kazumi 官方规则主口径。
    Xpath,
    /// 覆盖更广规则形态。
    Css,
}

/// 数据源 Provider 统一插槽。
///
/// 约：方法彼此独立可失败；Provider 不持有全局网络限速责任（应用级限速/脱敏在 core）；
/// `ref_id` 语义由 Provider 自行定义并对自身方法对称。
/// `async_trait`（装箱 future）：Provider 需要 `Box<dyn SourceProvider>` 列表在 UI 侧选用。
#[async_trait::async_trait]
pub trait SourceProvider: Send {
    /// 稳定幂等的显示名（规则页/选集页用）。
    fn name(&self) -> &str;
    /// 解析方言声明：UI 端可为兼容性提示。
    fn dialect(&self) -> SelectorDialect;

    async fn search(&self, keyword: &str) -> Result<Vec<SearchEntry>>;
    /// 单部作品：返回按集序的直链列表（index 单调，从 1 起）。
    async fn episodes(&self, ref_id: &str) -> Result<Vec<EpisodeSource>>;
}
