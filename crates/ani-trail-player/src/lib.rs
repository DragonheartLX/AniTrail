//! 番迹 (AniTrail) 播放器接口层。
//!
//! 定位：**播放引擎与上层的统一契约**。Slint 不做视频解码（规划 M6），
//! 引擎实现（mpv 子窗口 / ffmpeg 软管线 / WebView）在 D3 spike 定型前不排斥；
//! 本 crate 只定接口与数据形状，引擎实现按这些契约写即可替换。
//!
//! Slint 画面桥不在此 crate（决策：`ani-trail-core::media` 负责
//! `VideoFrame → slint::Image`），本 crate 保持 UI 无关。
//!
//! Anime4K 预留：以 [`upscale::UpscalePolicy`] 冻结接口；真正实现是 mpv 风格
//! GLSL shader 链，随 GPU 引擎落地（M6-1 spike 后），见 upscale.rs 顶部说明。

pub mod engine;
pub mod error;
pub mod frame;
pub mod source;
pub mod upscale;

#[cfg(test)]
mod tests;

pub use error::Error;
pub type Result<T, E = Error> = std::result::Result<T, E>;
