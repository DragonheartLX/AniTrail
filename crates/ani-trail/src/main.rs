slint::include_modules!();

mod app_state;
mod config;
mod logging;
mod router;
mod splash;
mod ui_i18n;

use ui_i18n::splash as i18n_splash;

fn main() -> anyhow::Result<()> {
    logging::init();
    let main_window = MainWindow::new()?;

    // M2：状态单例（事件循环线程驻留）+ 路由回调回接（go/back/滚动上报）。
    app_state::init_state(main_window.as_weak());
    {
        let router = Router::get(&main_window);
        router.on_go(|route| app_state::with_state(|s| s.router().go(route)));
        router.on_switch(|route| app_state::with_state(|s| s.router().switch(route)));
        router.on_back(|| app_state::with_state(|s| s.router().back()));
        router.on_save_scroll(|y| app_state::with_state(|s| s.router().save_scroll(y as f64)));
    }

    main_window.set_splash_slogan_text(i18n_splash::SLOGAN.into());
    main_window.set_splash_loading_text(i18n_splash::LOADING_CLIENT.into());
    main_window.set_splash_version_text(
        format!(
            "v{} · {}",
            env!("CARGO_PKG_VERSION"),
            i18n_splash::VERSION_LABEL,
        )
        .into(),
    );

    // M2 · 开屏进度走真实启动阶段（配置→客户端构建；M4 sync 阶段接入同一驱动），
    // 完成后自动进主页：后台线程驱动，事件经 invoke_from_event_loop 进 UI 线程。
    splash::drive(main_window.as_weak());

    main_window.run()?;
    Ok(())
}
