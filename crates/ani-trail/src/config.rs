//! M0-3 · 应用配置：TOML 读取 → `MirrorRouter`/`TlsPolicy` 注入路径打通。
//!
//! 路径：`ani-trail-platform::layout` 统一收敛——根目录 `ani-trail/` 下分类
//! （桌面 = `data_local_dir()/ani-trail`；Android = `FilesDir()/ani-trail`，M9-3 注入），
//! 配置文件 = `<root>/config/config.toml`。
//! 镜像链（docs/规划/02 / 06b D12）：默认 `api.bgmapi.com` → `api.bgm.tv` → 用户自加；
//! **链内 ECH 源优先级最高**（stable 重排，组内保序）。文件缺失/镜像空 → 默认链。
//!
//! ```toml
//! # config.toml
//! [[mirrors]]
//! label = "bgm.tv ECH"                 # 可空，缺省按 api_base 生成
//! api_base = "https://api.bgm.tv"
//! ech_config_b64 = "AEX...AB"          # 可空/缺省 = 默认 TLS
//! ```

use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use ani_trail_bangumi::client::{ua, BangumiClient};
use ani_trail_bangumi::mirror::Mirror;
use ani_trail_bangumi::rate_limit::RateLimiter;
use ani_trail_bangumi::tls::TlsPolicy;

/// 配置盘点结果：排序后的镜像链。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    mirrors: Vec<ResolvedMirror>,
}

/// 归一化的单镜像配置（= TlsPolicy 已就位）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMirror {
    pub label: String,
    pub api_base: String,
    pub tls: TlsPolicy,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    mirrors: Vec<RawMirror>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct RawMirror {
    #[serde(default)]
    label: String,
    api_base: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ech_config_b64: Option<String>,
}

/// 默认链（D12）：`api.bgmapi.com`（主，实测可达）→ `api.bgm.tv`（备）。
fn default_mirrors() -> Vec<ResolvedMirror> {
    vec![
        ResolvedMirror {
            label: "bgmapi 主源".into(),
            api_base: "https://api.bgmapi.com".into(),
            tls: TlsPolicy::Default,
        },
        ResolvedMirror {
            label: "api.bgm.tv 备源".into(),
            api_base: "https://api.bgm.tv".into(),
            tls: TlsPolicy::Default,
        },
    ]
}

impl ResolvedMirror {
    /// 归一化：label 空 → 按 api_base 域名生成；ECH 空串 → Default；
    /// api_base 必须 http(s):// 开头。
    fn normalize(raw: RawMirror) -> anyhow::Result<Self> {
        let ech = raw
            .ech_config_b64
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .map(TlsPolicy::Ech)
            .unwrap_or(TlsPolicy::Default);
        if !raw.api_base.starts_with("https://") && !raw.api_base.starts_with("http://") {
            anyhow::bail!("镜像 api_base 必须以 http(s):// 开头：{}", raw.api_base);
        }
        let label = match raw.label.trim() {
            "" => raw
                .api_base
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .split('/')
                .next()
                .unwrap_or(&raw.api_base)
                .to_owned(),
            other => other.to_owned(),
        };
        Ok(Self {
            label,
            api_base: raw.api_base.trim_end_matches('/').to_owned(),
            tls: ech,
        })
    }
}

impl AppConfig {
    // M5c 设置页增删排序消费；当前仅测试触达。
    #[allow(dead_code)]
    pub fn mirrors(&self) -> &[ResolvedMirror] {
        &self.mirrors
    }

    /// 解析 TOML 文本；读盘与解析分离，便于测试与设置页回写共用。
    pub fn parse(toml_text: &str) -> anyhow::Result<Self> {
        let raw: RawConfig = toml::from_str(toml_text)?;
        let mut resolved: Vec<ResolvedMirror> = raw
            .mirrors
            .into_iter()
            .map(ResolvedMirror::normalize)
            .collect::<anyhow::Result<_>>()?;

        if resolved.is_empty() {
            resolved = default_mirrors();
        } else {
            // 用户链为准（M5c 增删排序的可控语义；默认链只解首装零配置）。
            // 链内排序：ECH 源优先级最高（stable，组内保序）。
            resolved.sort_by_key(|m| !matches!(m.tls, TlsPolicy::Ech(_)));
        }
        // api_base 去重（首个保留）。
        let mut seen = std::collections::HashSet::new();
        resolved.retain(|m| seen.insert(m.api_base.clone()));
        Ok(Self { mirrors: resolved })
    }

    /// 读磁盘：文件不存在 → 默认链（首装零配置可用）。
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) => Self::parse(&text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self {
                mirrors: default_mirrors(),
            }),
            Err(e) => Err(e).context(format!("读取配置失败：{}", path.display())),
        }
    }

    /// 默认路径：`platform::layout(Host)` 单根收敛下的 `config/config.toml`
    /// （Android 需 FilesDir 注入，M9-3）。
    pub fn default_path() -> anyhow::Result<std::path::PathBuf> {
        Ok(
            ani_trail_platform::layout(ani_trail_platform::PlatformBase::Host)
                .context("平台存储布局不可得（Android 需注入 FilesDir，见 docs/规划/01 §5）")?
                .config_path,
        )
    }

    /// 写回（设置页「增删排序」落盘，M5c 消费）。
    #[allow(dead_code)]
    pub fn write_to(&self, path: &Path) -> anyhow::Result<()> {
        let raw = RawConfig {
            mirrors: self
                .mirrors
                .iter()
                .map(|m| RawMirror {
                    label: m.label.clone(),
                    api_base: m.api_base.clone(),
                    ech_config_b64: match &m.tls {
                        TlsPolicy::Ech(b64) => Some(b64.clone()),
                        TlsPolicy::Default => None,
                    },
                })
                .collect(),
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context(format!("创建配置目录失败：{}", parent.display()))?;
        }
        std::fs::write(path, toml::to_string(&raw)?)
            .context(format!("写入配置失败：{}", path.display()))
    }

    /// M0-3 验收动作：假配置 → 构造成功（无网络，仅建 HTTP 句柄）。
    pub fn build_client(&self) -> anyhow::Result<BangumiClient> {
        let mirrors = self
            .mirrors
            .iter()
            .map(|m| Mirror {
                label: m.label.clone(),
                api_base: m.api_base.clone(),
                tls: m.tls.clone(),
            })
            .collect();
        Ok(BangumiClient::new(
            ua::ANITRAIL,
            mirrors,
            Arc::new(RateLimiter::default()),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(s: &str) -> AppConfig {
        AppConfig::parse(s).expect("配置解析应成功")
    }

    /// 文件缺失语义（借空文本承载）→ 默认链， кроме用户链为空同样落默认。
    #[test]
    fn empty_config_falls_back_to_defaults() {
        let cfg = parse_ok("");
        assert_eq!(
            cfg.mirrors(),
            &[
                ResolvedMirror {
                    label: "bgmapi 主源".into(),
                    api_base: "https://api.bgmapi.com".into(),
                    tls: TlsPolicy::Default
                },
                ResolvedMirror {
                    label: "api.bgm.tv 备源".into(),
                    api_base: "https://api.bgm.tv".into(),
                    tls: TlsPolicy::Default
                },
            ]
        );
    }

    /// D12：用户自加排默认链之后；链内 ECH 源优先级最高（组内保序）。
    #[test]
    fn user_chain_wins_ech_first() {
        let cfg = parse_ok(
            r#"
            [[mirrors]]
            label = "worker 反代"
            api_base = "https://bgm.example.workers.dev"

            [[mirrors]]
            label = "ECH 通了"
            api_base = "https://api.bgm.tv"
            ech_config_b64 = "  AEXAB==  "

            [[mirrors]]
            label = ""
            api_base = "https://example.org/api/"
        "#,
        );
        let order: Vec<(&str, &TlsPolicy)> = cfg
            .mirrors()
            .iter()
            .map(|m| (m.label.as_str(), &m.tls))
            .collect();
        assert_eq!(
            order,
            [
                ("ECH 通了", &TlsPolicy::Ech("AEXAB==".into())), // ECH 最高优先；空白已被 trim
                ("worker 反代", &TlsPolicy::Default),
                ("example.org", &TlsPolicy::Default), // 空 label → 域名补全；尾斜杠已去
            ] // 用户把 ECH 挂到 api.bgm.tv 域；用户链为准，默认链不干涉
        );
    }

    /// 同域重复 → 去重不重复建句柄（首个保留）。
    #[test]
    fn duplicate_domain_dedup() {
        let cfg = parse_ok(
            r#"
            [[mirrors]]
            api_base = "https://api.bgmapi.com"

            [[mirrors]]
            label = "手工再来一遍"
            api_base = "https://api.bgmapi.com/"
        "#,
        );
        let bases: Vec<&str> = cfg.mirrors().iter().map(|m| m.api_base.as_str()).collect();
        assert_eq!(bases, ["https://api.bgmapi.com"]);
    }

    #[test]
    fn invalid_api_base_rejected() {
        assert!(AppConfig::parse(
            r#"
            [[mirrors]]
            api_base = "bgm.tv"
        "#
        )
        .is_err());
    }

    /// M0-3 验收：假配置 → client 构造成功（无网络）。
    #[test]
    fn fake_config_builds_client() {
        let cfg = parse_ok(
            r#"
            [[mirrors]]
            label = "worker 反代"
            api_base = "https://bgm.example.workers.dev"
        "#,
        );
        let client = cfg.build_client().expect("构造应成功");
        assert!(client.user_agent().starts_with("AniTrail/"));
        assert_eq!(client.user_agent(), ua::ANITRAIL);
    }

    /// 写回 → 重读 roundtrip 保序保 ECH。
    #[test]
    fn write_roundtrip_reload() {
        let cfg = AppConfig::parse(
            r#"
            [[mirrors]]
            label = "worker 反代"
            api_base = "https://bgm.example.workers.dev"
        "#,
        )
        .unwrap();
        let dir = std::env::temp_dir().join(format!("anitrail-m0-3-test-{}", std::process::id()));
        let path = dir.join("config.toml");
        cfg.write_to(&path).expect("写回应成功");
        let reloaded = AppConfig::load(&path).expect("重读应成功");
        assert_eq!(cfg.mirrors(), reloaded.mirrors());
        std::fs::remove_dir_all(dir).ok();
    }
}
