//! 接口契约测试（不联网/无真实引擎：引擎后端随 spike 接入）。

use std::time::Duration;

use crate::engine::PlaybackState;
use crate::frame::{FrameSink, VideoFrame};
use crate::upscale::UpscalePolicy;

/// 帧直通的演示性 sink——测试与后续 core 桥的参照物。
#[derive(Default)]
struct CollectSink {
    frames: Vec<VideoFrame>,
}

impl FrameSink for CollectSink {
    fn present(&mut self, frame: &VideoFrame) -> crate::Result<()> {
        self.frames.push(frame.clone());
        Ok(())
    }
}

#[test]
fn frame_入sink且记录() {
    let f = VideoFrame::new(2, 2, Some(Duration::from_millis(3)), vec![0; 16]).unwrap();
    let mut sink = CollectSink::default();
    sink.present(&f).unwrap();
    assert_eq!(sink.frames.len(), 1);
    assert_eq!(sink.frames[0].pts(), Some(Duration::from_millis(3)));
}

#[test]
fn policy_默认值() {
    assert_eq!(PlaybackState::Idle, crate::engine::PlaybackState::Idle);
    assert!(!UpscalePolicy::None.is_anime4k());
    assert!(UpscalePolicy::Anime4k(crate::upscale::Anime4kConfig::typical_order([])).is_anime4k());
}

#[test]
fn anime4k_空shader报错() {
    let err = crate::upscale::validate(&UpscalePolicy::Anime4k(
        crate::upscale::Anime4kConfig::typical_order([]),
    ))
    .expect_err("空 shader 应报错");
    assert!(err.to_string().contains("shader"), "{err}");
    assert!(crate::upscale::validate(&UpscalePolicy::None).is_ok());
}
