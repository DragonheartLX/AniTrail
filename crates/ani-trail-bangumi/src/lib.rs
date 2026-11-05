//! 番迹 (AniTrail) 的 Bangumi 对接层。
//!
//! 职责：
//! * [`client`] — 带 token 的 REST 客户端（User-Agent 规范、限速）；
//! * [`mirror`] — 多镜像降级路由（`api.bgm.tv` 主源失联时自动切换）；
//! * [`tls`] — TLS 策略，含 ECH（Encrypted Client Hello）预留；
//! * [`rate_limit`] — 客户端侧限速（默认 ≥1s/请求）；
//! * [`auth`] — 访问令牌（OAuth 凭证未注册前走手动 token）。
//!
//! ECH 说明：rustls 0.23（本仓库 `=0.23.46`）稳定版支持 `ClientConfig::builder().with_ech()`，
//! HPKE 套件仅由 `aws_lc_rs` provider 提供。reqwest 自身无 ECH API，
//! 故 ECH 镜像通过 `use_preconfigured_tls` 注入自定义 rustls 配置，
//! 两边 rustls 版本以 `=0.23.46` 精确对齐保证 downcast 成功。
//! ECHConfigList 来自该 host 的 DNS HTTPS(Type65) 记录 `ech` 参数的 base64 值
//! （DoH 查询在应用层完成，自动发现属后续工作，见 docs/AniTrail-实现规划.md D0.2）。

pub mod auth;
pub mod client;
pub mod error;
pub mod mirror;
pub mod models;
pub mod rate_limit;
pub mod tls;

#[cfg(test)]
mod tests;

pub use error::{Error, Result};
