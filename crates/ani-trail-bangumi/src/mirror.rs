//! 多镜像降级路由。
//!
//! 现状（2026-11-07 实测）：本网络直连 `api.bgm.tv` 失联（DNS 解析异常 + HTTPS 超时），
//! 镜像切换为刚需。镜像_sdk 由应用设置页提供（首装只有主源），支持每镜像独立 TLS 策略，
//! 健康镜像记入常用序；连接级失败进入冷却，恢复判定交给客户端每轮新尝试。

use crate::error::{Error, Result};
use crate::tls::{self, TlsPolicy};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 一个 Bangumi API 镜像：base URL + TLS 策略。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mirror {
    /// 展示名（设置页用），如 `bgm.tv 主源`。
    pub label: String,
    /// API 根，如 `https://api.bgm.tv`（无尾斜杠）。
    pub api_base: String,
    /// TLS 策略；ECH 镜像在此声明。
    pub tls: TlsPolicy,
}

impl Mirror {
    pub fn primary() -> Self {
        Self {
            label: "bgm.tv 主源".into(),
            api_base: "https://api.bgm.tv".into(),
            tls: TlsPolicy::Default,
        }
    }

    /// 拼接 API 路径。
    pub fn url(&self, path: &str) -> String {
        format!("{}/{}", self.api_base.trim_end_matches('/'), path.trim_start_matches('/'))
    }
}

pub struct MirrorRouter {
    handles: Vec<Handle>,
    /// 下次优先尝试的下标（最近健康镜像沿用）。
    preferred: usize,
    /// 处于冷却的镜像标签 -> 到期时刻。
    cooldown_until: HashMap<String, Instant>,
    cooldown_for: Duration,
}

/// 关于某个镜像的完整句柄（构建完成的 HTTP client）。
struct Handle {
    mirror: Mirror,
    http: reqwest::Client,
}

impl MirrorRouter {
    pub fn new(user_agent: &str, mirrors: Vec<Mirror>) -> Result<Self> {
        if mirrors.is_empty() {
            return Err(Error::Config("镜像列表为空，无法构建 Bangumi 客户端".into()));
        }
        let mut handles = Vec::with_capacity(mirrors.len());
        for m in mirrors {
            let base = reqwest::Client::builder().user_agent(user_agent.to_owned());
            let http = tls::apply(base, &m.tls)?.build().map_err(reqwest_err)?;
            handles.push(Handle { mirror: m, http });
        }
        Ok(Self {
            handles,
            preferred: 0,
            cooldown_until: HashMap::new(),
            cooldown_for: Duration::from_secs(5 * 60),
        })
    }

    pub fn mirrors(&self) -> Vec<&Mirror> {
        self.handles.iter().map(|h| &h.mirror).collect()
    }

    /// 观测/测试用：健康镜像标签序。
    pub fn healthy_labels(&self) -> Vec<&str> {
        let now = Instant::now();
        self.handles
            .iter()
            .map(|h| &h.mirror.label)
            .filter(|l| self.cooldown_until.get(*l).is_none_or(|t| *t <= now))
            .map(|l| l.as_str())
            .collect()
    }

    /// 按Preferred顺序尝试：首个收到响应的镜像直传 Ok（含 4xx/5xx，业务错误不换镜像）。
    /// 连接级失败（connect/timeout/tls）该镜像冷却 5 分钟后继续下一个。
    pub async fn send<F>(&mut self, make: F) -> Result<reqwest::Response>
    where
        F: Fn(&Mirror, &reqwest::Client) -> reqwest::RequestBuilder,
    {
        let n = self.handles.len();
        let mut last_err: Option<Error> = None;
        for offset in 0..n {
            let idx = (self.preferred + offset) % n;
            let handle = &self.handles[idx];
            let label = handle.mirror.label.clone();
            if self.cooldown_until.get(&label).is_some_and(|t| *t > Instant::now()) {
                continue;
            }
            match make(&handle.mirror, &handle.http).send().await {
                Ok(resp) => {
                    self.preferred = idx;
                    self.cooldown_until.remove(&label);
                    return Ok(resp);
                }
                Err(e) if e.is_connect() || e.is_timeout() => {
                    self.cooldown_until
                        .insert(label, Instant::now() + self.cooldown_for);
                    last_err = Some(Error::Transport(e.to_string()));
                }
                Err(e) => return Err(Error::Transport(e.to_string())),
            }
        }
        Err(last_err.unwrap_or(Error::AllMirrorsUnreachable {
            detail: "全部镜像在冷却期".into(),
        }))
    }

    /// 强制重置冷却（用户手动“重试”按钮时调用）。
    pub fn reset_cooldowns(&mut self) {
        self.cooldown_until.clear();
    }
}

fn reqwest_err(e: reqwest::Error) -> Error {
    Error::Config(format!("HTTP 客户端构建失败: {e}"))
}

#[cfg(test)]
pub(crate) fn rotation_order(preferred: usize, n: usize) -> Vec<usize> {
    (0..n).map(|o| (preferred + o) % n).collect()
}
