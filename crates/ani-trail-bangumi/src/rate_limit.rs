//! 客户端侧限速：全局最小请求间隔（Bangumi 建议约 1s/请求）。
//!
//! 跨镜像同样受控——切换镜像不放松速率，避免封禁级联。

use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// 刻意不做成内部执守 IntervalGuard：未来多 client 共享一个 limiter 时按此单例注入。
#[derive(Debug)]
pub struct RateLimiter {
    min_interval: Duration,
    next_ok_at: Mutex<Option<Instant>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

impl RateLimiter {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            min_interval,
            next_ok_at: Mutex::new(None),
        }
    }

    /// 等到本请求的放行时刻。占用下一个槽位（先占位再等待，保证共调用方排队不重排）。
    pub async fn acquire(&self) {
        let now = Instant::now();
        let wait = {
            let mut slot = self.next_ok_at.lock().await;
            let target = match *slot {
                Some(t) if t > now => t,
                _ => now,
            };
            // 先写入占用，再让出 guard 等待
            *slot = Some(target + self.min_interval);
            target - now
        };
        if wait > Duration::ZERO {
            tokio::time::sleep(wait).await;
        }
    }

    #[cfg(test)]
    pub fn min_interval(&self) -> Duration {
        self.min_interval
    }
}
