//! 核心桥接测试。

use ani_trail_player::frame::{FrameSink, VideoFrame};
use std::time::Duration;

#[test]
fn frame_to_slint_image_size() {
    let f = VideoFrame::new(4, 3, Some(Duration::from_millis(5)), vec![0u8; 4 * 3 * 4]).unwrap();
    let img = crate::media::frame_to_slint_image(&f);
    let s = img.size();
    assert_eq!((s.width, s.height), (4, 3));
}

#[test]
fn sink_empty_then_peek_after_push() {
    let sink = crate::media::SlintFrameSink::new();
    assert!(sink.peek().is_none());
    let f = VideoFrame::new(2, 2, None, vec![1u8; 16]).unwrap();
    let mut sink = sink;
    sink.present(&f).unwrap();
    let img = sink.peek().expect("投帧后应有");
    let size = img.size();
    assert_eq!((size.width, size.height), (2u32, 2u32));
}
