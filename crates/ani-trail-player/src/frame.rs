//! 帧数据形状：引擎 → 画面 advice 的统一交付物。
//!
//! RGBA8、紧 stride（每行 `width*4`）；pts 可缺失（纯音频场景）。
//! 任意引擎实现与任何上层（core 侧 Slint 桥）都按此契约交接。

use std::time::Duration;

use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoFrame {
    width: u32,
    height: u32,
    pts: Option<Duration>,
    rgba: Vec<u8>,
}

impl VideoFrame {
    /// 构建并校验长度 == width*height*4。
    pub fn new(width: u32, height: u32, pts: Option<Duration>, rgba: Vec<u8>) -> Result<Self> {
        let need = (width as u64)
            .checked_mul(height as u64)
            .and_then(|px| px.checked_mul(4))
            .ok_or_else(|| Error::Frame("帧像素数溢出".into()))?;
        if rgba.len() as u64 != need {
            return Err(Error::Frame(format!(
                "RGBA 长度 {rgba_len} ≠ {w}x{h}x4 ({need})",
                rgba_len = rgba.len(),
                w = width,
                h = height
            )));
        }
        Ok(Self {
            width,
            height,
            pts,
            rgba,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pts(&self) -> Option<Duration> {
        self.pts
    }

    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    pub fn into_rgba(self) -> Vec<u8> {
        self.rgba
    }

    /// 尺寸变化（换集/达奈转换：Anime4K 放大后的帧）场合的便捷重建。
    pub fn resized(
        self,
        new_width: u32,
        new_height: u32,
        new_rgba: Vec<u8>,
        new_pts: Option<Duration>,
    ) -> Result<Self> {
        Self::new(new_width, new_height, new_pts, new_rgba)
    }
}

/// 画面交付口。core 侧的 Slint 桥实现它；引擎只能看到此契约。
///
/// 不带 `Send`：sink 由创建它的线程持有（UI 线程），引擎产出帧交给桥调度。
pub trait FrameSink {
    fn present(&mut self, frame: &VideoFrame) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_length_checks() {
        assert!(VideoFrame::new(2, 2, None, vec![0; 16]).is_ok());
        assert!(matches!(
            VideoFrame::new(2, 2, None, vec![0; 15]),
            Err(Error::Frame(m)) if { m.contains("≠") }
        ));
        assert!(VideoFrame::new(4, 4, None, vec![0; 16]).is_err());
    }

    #[test]
    fn frame_resize_shares_pts() {
        let f = VideoFrame::new(2, 2, Some(Duration::from_millis(7)), vec![7; 16]).unwrap();
        let pts = f.pts();
        let g = f.resized(4, 4, vec![9; 64], pts).expect("放大后形状合法");
        assert_eq!(
            (g.width(), g.height(), g.pts()),
            (4, 4, Some(Duration::from_millis(7)))
        );
        assert_eq!(g.rgba().len(), 64);
    }
}
