// M0-2（按用户裁定收缩）：release 仅隐藏控制台（windows_subsystem）；
// panic hook 原生弹窗不做——默认 hook 打印保留，桌面运行由终端/系统托管
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod config;
mod logging;
mod ui_i18n;

use ui_i18n::splash;

fn main() -> anyhow::Result<()> {
    logging::init();
    // M0-3：配置 → MirrorRouter/TlsPolicy 注入路径打通；M4 起 client 归 core sync、M2 起 AppState 转接。
    // 无配置宿主（Android 沙箱无 $HOME/XDG，M9-3 落 FilesDir 分支）→ 默认链内存配置，不落盘。
    let app_config = match config::AppConfig::default_path() {
        Ok(path) => config::AppConfig::load(&path)?,
        Err(_) => config::AppConfig::parse("")?,
    };
    let _bangumi = app_config.build_client()?;
    let main_window = MainWindow::new()?;
    main_window.set_splash_slogan_text(splash::SLOGAN.into());
    main_window.set_splash_loading_text(splash::LOADING_BANGUMI.into());
    main_window.set_splash_version_text(
        format!("v{} · {}", env!("CARGO_PKG_VERSION"), splash::VERSION_LABEL,).into(),
    );
    main_window.run()?;
    Ok(())
}
