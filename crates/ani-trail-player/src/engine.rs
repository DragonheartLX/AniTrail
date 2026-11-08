//! 解码引擎统一接口 + Anime4K 预留。
//!
//! 契约决策（本 crate 不含解码）：
//! * UI（core 侧桥接）只认 [`VideoEngine`]/[`FrameSink`]/[`PlaybackEvent`]；
//! * 引擎实现（mpv / ffmpeg / …）在 D3 spike 后按需选择，落在此 trait 的 impl 上；
//! * Anime4K 经 [`UpscalePolicy`] 冻结接口：mpv 风格 GLSL shader 文件链，
//!   引擎负责按自身能力热替换或预挂钩，本 crate 不实现像素处理。

use std::time::Duration;

use crate::error::Result;
use crate::frame::VideoFrame;
use crate::source::Source;
use crate::upscale::UpscalePolicy;

/// 播放状态机（引擎内部管理，事件驱动 UI）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    /// 已 build、未 load。
    Idle,
    /// load 中（元数据/容器解析）。
    Loading,
    Playing,
    Paused,
    /// 无数据可用（网络卡顿/换集间隙）。
    Stalled,
    /// 当前条目播完（连播下一个由引擎依据 next_source() 自启）。
    Ended,
}

/// 引擎投递给上层的事件流。`poll_event` 由 UI 事件循环或后台 task 周期驱动。
#[derive(Clone, Debug)]
pub enum PlaybackEvent {
    /// 周期进度（引擎自行保证节奏，建议 ≥4Hz）。
    Progress {
        position: Duration,
        total: Duration,
    },
    StateChanged(PlaybackState),
    /// 缓冲进度 0.0..=1.0（仅 Stalled 期间有意义）。
    Buffer(f32),
    Ended,
    Error(String),
}

/// 媒体描述。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MediaInfo {
    pub duration: Option<Duration>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub audio_streams: u32,
}

pub trait VideoEngine: Send {
    /// 载入一个源，返回元信息；楼 playlists 应先 `load` 再 play。
    fn load(&mut self, source: &Source) -> Result<MediaInfo>;
    fn play(&mut self) -> Result<()>;
    fn pause(&mut self) -> Result<()>;
    fn is_playing(&self) -> bool;

    /// 绝对定位；引擎必须紧迫态（不因 seek 暂停或重置 Ended）。
    fn seek(&mut self, position: Duration) -> Result<()>;
    fn position(&self) -> Duration;

    /// 音量 0.0..=1.0、倍速 0.05..=10.0；静音不影响音量取值。
    fn set_volume(&mut self, volume: f32) -> Result<()>;
    fn set_speed(&mut self, speed: f32) -> Result<()>;
    fn set_muted(&mut self, mute: bool) -> Result<()>;

    /// 连播挂点：引擎在 Ended 时如已设置且允许自动连播则加载下一个。
    fn set_next(&mut self, next: Option<Source>) -> Result<()>;

    /// Anime4K 等。设置后必须从下一个已交付帧起生效；实现允许异步切换，
    /// 但切换期间不允许输出未处理的原始帧（要么旧策略要么新策略，不应混帧）。
    fn set_upscale_policy(&mut self, policy: UpscalePolicy) -> Result<()>;

    /// 拉取事件。上层有义务定期调用，否则积压事件可被丢弃。
    fn poll_event(&mut self) -> Option<PlaybackEvent>;

    /// 取当前可用帧（合成/解码最新画面）。播放暂停时仍应可返回最后一帧。
    fn next_frame(&mut self) -> Result<Option<VideoFrame>>;

    /// 释放资源并停止渲染。引擎实现必须幂等。
    fn stop(&mut self) -> Result<()>;
}
