//! 数据源层统一错误。

use thiserror::Error;

/// Provider 统一错误形态。具体原因（站点/规则）由 impl 以 [`Error::Upstream`] 带出。
#[derive(Debug, Error)]
pub enum Error {
    /// 站点/规则侧失败（网络、解析空结果、页面改版）。
    #[error("数据源「{provider}」失败: {detail}")]
    Upstream { provider: String, detail: String },

    /// 上层填入的凭据/引用无效（如 BT Provider 收到非磁力 hash）。
    #[error("无效的来源引用: {0}")]
    InvalidRef(String),

    /// Provider 配置缺失（用户未装规则/未配置下载目录等）。
    #[error("数据源未配置: {0}")]
    Unconfigured(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
