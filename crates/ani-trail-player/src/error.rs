//! 统一错误。

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 引擎实现自身的运行时失败（具体文案由实现给出）。
    #[error("播放引擎错误: {0}")]
    Engine(String),

    /// 媒体源不可用/格式不支持。
    #[error("媒体源无效: {0}")]
    SourceInvalid(String),

    /// 帧数据形状错误（RGBA 长度与宽高不符等）。
    #[error("帧数据错误: {0}")]
    Frame(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
