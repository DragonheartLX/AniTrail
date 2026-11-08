//! M0-4 · `tracing` 初始化 + 日志脱敏基线（docs/规划/08 §4-1 / D21）。
//!
//! 红线：token/Authorization、镜像整套 URL、片源站点 URL 一律不打——敏感值只允许
//! 经 [`Secret`] 进日志（Display/Debug 恒为屏蔽字）；镜像/片源 URL 一律不打，
//! 区分镜像用 `Mirror.label`（用户展示名）。字段化优于消息插值（span 属性）。
//!
//! 初始化：`RUST_LOG` 控制级别；缺省 = 全库 info + 传输库噪声降 warn，避免全量 debug 拖 UI 线程。

use std::fmt;

use tracing_subscriber::EnvFilter;

/// 全局 subscriber；重复调用无害（只生效一次）。
pub fn init() {
    const DEFAULT: &str = "info,\
        hyper=warn,reqwest=warn,rustls=warn,h2=warn,\
        winit=warn,slint=warn,ani_trail_platform=warn";
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

/// 敏感值包装：Display/Debug 恒呈现屏蔽字，任何日志设施都带不出原值。
/// 用法：`info!(auth = %Secret::new(token), mirror = mirror.label)`。
// M4 起各同步/抓取线消费；当前 logging 基线仅测试触达。
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Secret<T>(T);

#[allow(dead_code)]
impl<T> Secret<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
    /// 取值仅限业务路径内部使用；落日志即触红线（评审检查项）。
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl<T> std::ops::Deref for Secret<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tracing::info;
    use tracing_subscriber::fmt::MakeWriter;

    const TOKEN: &str = "Bearer faketesttoken";
    const MIRROR_URL: &str = "https://bgm.example.workers.dev/v0/me?private=hush";

    /// 共享内存 writer：捕获 fmt 层输出做断言。
    struct Capture(Arc<Mutex<Vec<u8>>>);

    impl<'a> MakeWriter<'a> for Capture {
        type Writer = CaptureWrite<'a>;

        fn make_writer(&'a self) -> CaptureWrite<'a> {
            CaptureWrite(self.0.lock().unwrap())
        }
    }

    struct CaptureWrite<'a>(std::sync::MutexGuard<'a, Vec<u8>>);

    impl std::io::Write for CaptureWrite<'_> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// 红线验收（05 M0-4 断言日志无 token/URL 字段）：Secret 包裹值以屏蔽字呈现，
    /// token 明文与整套镜像 URL/域名不得现身；展示名（mirror_label）可正常携带。
    #[test]
    fn secret_and_url_never_leak_to_logs() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_writer(Capture(Arc::clone(&buffer)))
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            info!(
                auth = %Secret::new(TOKEN),
                mirror_label = "worker 反代", // 展示名，非 URL
                normal = "plain text ok"
            );
            // 错误路同样只经包装（Q0-4 红线示例）。
            info!(reason = "unauthorized", token = %Secret::new(TOKEN));
        });
        let out = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
        assert!(!out.contains("faketesttoken"), "token 明文落日志：{out}");
        assert!(!out.contains(MIRROR_URL), "镜像整套 URL 落日志：{out}");
        assert!(
            !out.contains("bgm.example.workers.dev"),
            "镜像域名落日志：{out}"
        );
        assert!(out.contains("[REDACTED]"));
        assert!(out.contains("plain text ok"), "正常字段被误屏蔽：{out}");
    }
}
