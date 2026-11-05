//! Bangumi 客户端：镜像路由 + 限速 + token + UA 规范。
//!
//! Bangumi API 使用要求统一 UA，格式 `App名/版本 (主页或邮箱)`；
//! 客户端每次请求限速默认 ≥1s（可换 RateLimiter 定制）。

use crate::auth::AccessToken;
use crate::error::{Error, Result};
use crate::mirror::{Mirror, MirrorRouter};
use crate::models::{Me, Subject};
use crate::rate_limit::RateLimiter;
use std::sync::Arc;

pub mod ua {
    /// 番迹的默认 User-Agent；版本号随 workspace 版本动态注入。
    pub const ANITRAIL: &str = concat!(
        "AniTrail/",
        env!("CARGO_PKG_VERSION"),
        " (https://github.com/DragonheartLX/AniTrail)"
    );
}

pub struct BangumiClient {
    router: tokio::sync::Mutex<MirrorRouter>,
    limiter: Arc<RateLimiter>,
    token: tokio::sync::RwLock<Option<AccessToken>>,
    user_agent: String,
}

impl BangumiClient {
    /// `mirrors` 顺序即优先级；EOF 会自动尝试下一个、健康镜像被记住。
    pub fn new(user_agent: &str, mirrors: Vec<Mirror>, limiter: Arc<RateLimiter>) -> Result<Self> {
        Ok(Self {
            router: tokio::sync::Mutex::new(MirrorRouter::new(user_agent, mirrors)?),
            limiter,
            token: tokio::sync::RwLock::new(None),
            user_agent: user_agent.to_owned(),
        })
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub async fn set_token(&self, token: Option<AccessToken>) {
        *self.token.write().await = token;
    }

    pub async fn token(&self) -> Option<AccessToken> {
        self.token.read().await.clone()
    }

    /// 首装无网络镜像失联时的手动恢复。
    pub async fn reset_network(&self) {
        self.router.lock().await.reset_cooldowns();
    }

    /// `GET /v0/me` —— 需登录。
    pub async fn me(&self) -> Result<Me> {
        let Some(token) = self.token().await else {
            return Err(Error::Unauthorized);
        };
        self.get_json("/v0/me", Some(&token)).await
    }

    /// `GET /v0/subjects/{id}` —— 公开接口。
    pub async fn subject(&self, id: u32) -> Result<Subject> {
        let path = format!("/v0/subjects/{id}");
        self.get_json(&path, None).await
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        token: Option<&AccessToken>,
    ) -> Result<T> {
        self.limiter.acquire().await;
        let resp = {
            let mut router = self.router.lock().await;
            router
                .send(|mirror, http| {
                    let mut b = http.get(mirror.url(path));
                    if let Some(t) = token {
                        b = b.header(reqwest::header::AUTHORIZATION, t.to_header_value());
                    }
                    b
                })
                .await?
        };
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                body: body.chars().take(600).collect(),
            });
        }
        resp.json().await.map_err(|e| Error::Transport(e.to_string()))
    }
}
