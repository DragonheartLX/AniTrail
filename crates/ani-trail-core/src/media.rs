//! Slint 画面桥：引擎出来的 [`VideoFrame`] → UI `Image` 属性。
//!
//! 决策依据：Slint 不做视频解码，`ani-trail-player` 保持 UI 无关，
//! Slint 侧拼接在此（见 docs/AniTrail-实现规划.md“player不用对接slint,这个由core负责”）。
//!
//! 用法：UI 声明 `in property <image> video-frame;`，桥在 Slint 事件循环里
//! `set_video_frame(sink.peek().unwrap_or_default())` 刷新。

use std::cell::RefCell;
use std::rc::Rc;

use ani_trail_player::frame::{FrameSink, VideoFrame};

/// 帧转 `slint::Image`（RGBA8 紧 stride）。
pub fn frame_to_slint_image(frame: &VideoFrame) -> slint::Image {
    let pixels = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
        frame.rgba(),
        frame.width(),
        frame.height(),
    );
    slint::Image::from_rgba8(pixels)
}

/// 维护最新一帧的 sink。UI 桥轮询/事件回调里取走显示。
///
/// 多线程注意：引擎线程 present，UI 事件循环 read——Slint 标准做法是把
/// 交换动作 `invoke_from_event_loop` 收口到 UI 线程，本类型不跨线程加锁，
/// 供单线程（UI 线程侧）驱动；引擎线程侧只产出帧交给桥调度，不直接用它。
#[derive(Clone, Default)]
pub struct SlintFrameSink {
    latest: Rc<RefCell<Option<slint::Image>>>,
}

impl SlintFrameSink {
    pub fn new() -> Self {
        Self::default()
    }

    /// 最近一帧；未收到帧时 `None`（UI 侧保持骨架/占位）。
    pub fn peek(&self) -> Option<slint::Image> {
        self.latest.borrow().clone()
    }
}

impl FrameSink for SlintFrameSink {
    fn present(&mut self, frame: &VideoFrame) -> ani_trail_player::Result<()> {
        *self.latest.borrow_mut() = Some(frame_to_slint_image(frame));
        Ok(())
    }
}

pub use ani_trail_player;
