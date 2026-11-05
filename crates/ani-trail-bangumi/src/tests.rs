//! 单元测试（不联网：真实链路需镜像联通后联调）。

use std::time::Duration;
use std::str::FromStr;

use crate::auth::AccessToken;
use crate::error::Error;
use crate::mirror::{rotation_order, Mirror};
use crate::rate_limit::RateLimiter;
use crate::tls::TlsPolicy;

#[tokio::test]
async fn rate_limiter_隔开两次请求() {
    let lim = RateLimiter::new(Duration::from_millis(200));
    lim.acquire().await;
    let t0 = std::time::Instant::now();
    lim.acquire().await;
    assert!(t0.elapsed() >= Duration::from_millis(200));
}

#[test]
fn rotation_从preferred开始环形() {
    assert_eq!(rotation_order(0, 3), vec![0, 1, 2]);
    assert_eq!(rotation_order(2, 3), vec![2, 0, 1]);
}

#[test]
fn tls_policy_宽容解析() {
    assert_eq!(TlsPolicy::from_optional_b64(None), TlsPolicy::Default);
    assert_eq!(TlsPolicy::from_optional_b64(Some("   ")), TlsPolicy::Default);
    assert_eq!(
        TlsPolicy::from_optional_b64(Some("AAAA")),
        TlsPolicy::Ech("AAAA".into())
    );
}

#[test]
fn token_归一化粘贴值() {
    assert_eq!(
        AccessToken::from_str("  Bearer abc123  ")
            .unwrap()
            .0,
        "abc123"
    );
    assert!(AccessToken::from_str("").is_err());
}

#[test]
fn ua_版本动态注入() {
    let ua = crate::client::ua::ANITRAIL;
    let expected = concat!("AniTrail/", env!("CARGO_PKG_VERSION"));
    assert!(ua.starts_with(expected), "{ua}");
    assert!(ua.ends_with("(https://github.com/DragonheartLX/AniTrail)"), "{ua}");
}

#[tokio::test]
async fn mirror_非法ech在构建期报错() {
    // base64 可解码但内容非法：EchConfig::new 校验失败 → TlsConfig 错误
    let m = Mirror {
        label: "试".into(),
        api_base: "https://api.bgm.tv".into(),
        tls: TlsPolicy::Ech("AAAAAAAAAAAAAAAAAAAAAA==".into()),
    };
    let err = match crate::mirror::MirrorRouter::new("AniTrail/dev", vec![m]) {
        Err(e) => e,
        Ok(_) => panic!("非法 ECH 配置应当导致构建失败"),
    };
    assert!(err.to_string().contains("ECH"), "{err}");
}

#[tokio::test]
async fn mirror_空列表报错() {
    let err = match crate::mirror::MirrorRouter::new("AniTrail/dev", vec![]) {
        Err(e) => e,
        Ok(_) => panic!("空镜像列表应当失败"),
    };
    assert!(matches!(err, Error::Config(_)), "{err}");
}
