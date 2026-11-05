//! 访问令牌。
//!
//! 决策 D4.1：OAuth 凭证未注册前提供手动 token 降级；OAuth 全流程（PKCE + 本地回环）
//! 在 M4-2 落地本模块，`AccessToken` 是届时唯一被上层消费的定型，故先收口在此。

/// Bangumi v0 API 的 Bearer 访问令牌。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessToken(pub String);

impl AccessToken {
    /// `Authorization: Bearer ???` 头。
    pub fn to_header_value(&self) -> String {
        format!("Bearer {}", self.0)
    }
}

/// 归一化粘贴的 token：去空白、引号、`Bearer ` 前缀。
impl std::str::FromStr for AccessToken {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let t = s
            .trim()
            .trim_start_matches("Bearer ")
            .trim_start_matches("bearer ")
            .trim_matches('"')
            .trim();
        if t.is_empty() {
            return Err(crate::error::Error::Config("访问令牌为空".into()));
        }
        Ok(AccessToken(t.to_owned()))
    }
}
