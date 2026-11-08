//! M2 · 开屏驱动：进度随真实后端阶段推进，完成即自动进主页（docs/规划/01 §2）。
//!
//! 数据流：后台线程串行执行启动阶段（真实工作：配置解析 → Bangumi 客户端构建），
//! 进度/文案经 `slint::invoke_from_event_loop` 进 UI 线程更新；100% 后一次性投递
//! `Router.go(Home)`——路由栈同步（Vector) 在 UI 线程的 app_state 内。
//! M4 同步管线接手后新增 `Sync`/`ImageWarmup` 阶段，走同一条 publishes 通道。
//!
//! 点按跳过仍有效（用户可先于完成手动进主页；后台阶段继续跑完不产生二次跳转）。

use std::thread;
use std::time::Duration;

use slint::Weak;

use crate::ui_i18n::splash;
use crate::{app_state, MainWindow, Route};

/// 后台驱动：串行真实工作 + 经事件循环回填。
pub fn drive(window: Weak<MainWindow>) {
    thread::spawn(move || {
        // 阶段 0：清零（0%）。
        post(&window, 0.0, crate::ui_i18n::splash::LOADING_CONFIG);

        // 阶段 1（0→40%）：配置解析/加载（真实工作）。
        let app_config = {
            match crate::config::AppConfig::default_path() {
                Ok(path) => crate::config::AppConfig::load(&path)
                    .unwrap_or_else(|_| crate::config::AppConfig::parse("").expect("默认链可解析")),
                // 无配置宿主（Android FilesDir 未注入，M9-3 判）→ 默认链。
                Err(_) => crate::config::AppConfig::parse("").expect("默认链可解析"),
            }
        };
        post(&window, 0.4, crate::ui_i18n::splash::LOADING_CONFIG);

        // 阶段 2（40→90%）：Bangumi 客户端构建（镜像句柄 + TLS/ECH 配置校验；无网络）。
        thread::sleep(Duration::from_millis(60)); // 让首帧进度可见
        let client = app_config
            .build_client()
            .expect("镜像客户端构建失败：查 config.toml（D36）");
        post(&window, 0.9, splash::LOADING_CLIENT);

        // 阶段 3（90→100%）：完成 → 投递路由 go(Home)。
        post(&window, 1.0, splash::LOADING_CLIENT);
        thread::sleep(Duration::from_millis(120)); // 100% 停留一拍
        drop(client);
        let _ = slint::invoke_from_event_loop(move || {
            app_state::with_state(|s| s.router().switch(Route::Home));
        });
    });
}

/// 设置进度与文案（UI 线程内执行）。
fn post(window: &Weak<MainWindow>, progress: f64, text: &'static str) {
    let win = window.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(win) = win.upgrade() {
            win.set_splash_progress(progress as f32);
            win.set_splash_loading_text(text.into());
        }
    });
}
