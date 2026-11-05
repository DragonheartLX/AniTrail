//! 统一错误类型。

/// 对接层统一结果。
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 所有镜像都打不通（连接被拒/超时/TLS 握手失败）。
    #[error("所有镜像不可达（最近错误：{detail}）")]
    AllMirrorsUnreachable { detail: String },

    /// ECH/TLS 配置非法（base64 解码失败、无兼容 HPKE 套件、版本不支持等）。
    /// 在 [`crate::mirror::MirrorRouter::new`] 构建期即暴露，避免运行期静默降级。
    #[error("TLS/ECH 配置非法: {0}")]
    TlsConfig(String),

    /// Bangumi 返回非 2xx，携带状态码与响应体片段。
    #[error("Bangumi API 返回 {status}: {body}")]
    Api { status: u16, body: String },

    /// 传输层错误（连接、超时、TLS 握手）。
    #[error("网络传输错误: {0}")]
    Transport(String),

    /// 响应体解析失败。
    #[error("响应解析失败: {0}")]
    Parse(#[from] serde_json::Error),

    /// 该接口需要登录但 client 未携带 token。
    #[error("未登录：此接口需要访问令牌")]
    Unauthorized,

    /// 镜像配置为空等初始化错误。
    #[error("{0}")]
    Config(String),
}
