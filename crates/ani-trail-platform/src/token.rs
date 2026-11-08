//! 令牌存储契约：访问令牌永不进明文配置（docs/规划/01 §5）。
//!
//! 平台 impl 落点（先契约后接线，禁止 stub）：
//! - 桌面：crate `keyring`（条目默认 `ani-trail`）—— **M4-4**（docs/规划/TODO M4 数据层）
//! - Android：`EncryptedSharedPreferences` 代 keyring —— **M9-3**
//!
//! 令牌值不进日志（`08` §4 脱敏红线；调用侧不得把返回值打日志）。

/// 访问令牌存取（账号维度，同步小体量 KV）。
pub trait TokenStore: Send + Sync {
    /// 取令牌；无凭据返回 Ok(None)（首装/未登录）。
    fn get(&self, account: &str) -> Result<Option<String>, TokenError>;
    /// 写/更新令牌（覆盖旧值，原子语义由 impl 保证）。
    fn set(&self, account: &str, token: &str) -> Result<(), TokenError>;
    /// 删除令牌（登出）；不存在视为成功。
    fn delete(&self, account: &str) -> Result<(), TokenError>;
}

/// 令牌存储错误：OS 侧拒绝（锁屏/权限/凭据服务不可用）等，不携带令牌明文。
#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("令牌存储不可用：{0}")]
    Unavailable(String),
    #[error("令牌存储拒绝（权限/平台态）")]
    Denied,
}
