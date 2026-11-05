//! TLS 策略：默认走 reqwest 内建 rustls；ECH 镜像注入自定义 rustls 配置。
//!
//! 严格要求：本 crate 的 `rustls` 依赖 `=0.23.46`，与 reqwest 0.12 内部一致，
//! 否则 `use_preconfigured_tls` 的 downcast 会失败。

use base64::engine::general_purpose::STANDARD as B64_PADDED;
use base64::engine::general_purpose::STANDARD_NO_PAD as B64_UNPADDED;
use base64::Engine as _;
use rustls::pki_types::EchConfigListBytes;
use std::sync::Arc;

use crate::error::{Error, Result};

/// 单个镜像的 TLS 策略。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TlsPolicy {
    /// reqwest 内建 rustls（ring provider，webpki roots）。对大多数镜像足够。
    Default,
    /// 启用 ECH（Encrypted Client Hello）：`base64` 编码的 ECHConfigList。
    ///
    /// 来源：对该 host 做 DNS HTTPS(Type65) 记录查询（推荐 DoH），取 `ech` 参数
    /// 原始字节并 base64 编码。配置在镜像构建期校验，非法即报错，不静默降级。
    Ech(String),
}

impl TlsPolicy {
    /// 从设置值构造：trim 空白，空值归 `Default`。
    pub fn from_optional_b64(s: Option<&str>) -> Self {
        match s.map(str::trim).filter(|s| !s.is_empty()) {
            Some(enc) => TlsPolicy::Ech(enc.to_owned()),
            // None | Some("") 都当默认 TLS 处理
            _ => TlsPolicy::Default,
        }
    }
}

/// 将 TLS 策略应用到 reqwest builder 上。
pub fn apply(
    builder: reqwest::ClientBuilder,
    policy: &TlsPolicy,
) -> Result<reqwest::ClientBuilder> {
    match policy {
        TlsPolicy::Default => Ok(builder),
        TlsPolicy::Ech(ech_b64) => {
            let config = rustls_config_for_ech(ech_b64)?;
            Ok(builder.use_preconfigured_tls(config))
        }
    }
}

/// 构建启用 ECH 的 rustls `ClientConfig`。
///
/// 注意 rustls 的 ECH 配置是 per-host 的：config list 必须来自目标 host 自身的
/// DNS 记录；跨 host 复用不会生效（握手将不加密 SNI，等价无 ECH）。
fn rustls_config_for_ech(ech_b64: &str) -> Result<rustls::ClientConfig> {
    let raw = decode_b64(ech_b64)?;
    let ech_config = rustls::client::EchConfig::new(
        EchConfigListBytes::from(raw),
        // 需求硬事实：rustls 的 ring provider 不含 HPKE，ECH 套件只在 aws_lc_rs provider
        rustls::crypto::aws_lc_rs::hpke::ALL_SUPPORTED_SUITES,
    )
    .map_err(|e| Error::TlsConfig(format!("ECH 配置不可用: {e}")))?;

    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    Ok(rustls::ClientConfig::builder_with_provider(provider)
        .with_ech(ech_config.into())
        .map_err(|e| Error::TlsConfig(format!("应用 ECH 失败: {e}")))?
        .with_root_certificates(root_store())
        .with_no_client_auth())
}

/// 解析 base64：容忍换行/空格，兼容有无 padding。
fn decode_b64(s: &str) -> Result<Vec<u8>> {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let trimmed = cleaned.trim_end_matches('=');
    B64_PADDED
        .decode(trimmed)
        .or_else(|_| B64_UNPADDED.decode(trimmed))
        .map_err(|e| Error::TlsConfig(format!("ECHConfigList base64 解码失败: {e}")))
}

fn root_store() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    roots
}
