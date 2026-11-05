//! 画面修饰策略：Anime4K 接口冻结。
//!
//! 决策：Anime4K 官方形态是 mpv 的 `--glsl-shader` GLSL 链
//! （ip-net-CNN 算子，运行于 GPU；Anime4K v4 提供 Restore/UpscaleCNN/Deblur/CA
//! 各类 shader 文件）。它依赖底层有 GPU 纹理管线的引擎。
//!
//! 本 crate 只冻结配置与策略选择：
//! * `None` — 原样输出（默认，CPU 软件管线也能播）；
//! * `Anime4k` — shader 文件列表 + 放大倍率，引擎按各自方式热路由；
//!   mpv 系引擎直接挂 `--glsl-shader=...`（Anime4K 语义），软件引擎启用时必须
//!   明确报错"该引擎不支持"，禁止静默丢弃策略。
//!
//! 像素处理实现不落在本 crate（软件 CNN 不实时、且重复造轮子）。

use std::path::PathBuf;

use crate::error::Result;

/// 上层对画面修饰的策略声明。
#[derive(Clone, Debug, PartialEq)]
pub enum UpscalePolicy {
    None,
    Anime4k(Anime4kConfig),
}

impl UpscalePolicy {
    pub fn is_anime4k(&self) -> bool {
        matches!(self, Self::Anime4k(_))
    }
}

/// Anime4K 配置。`shader_files` 按挂载顺序排列——Anime4K 效果由 shader 的
/// 串联次序决定（如 `Upscale/Denoise/Deblur/Restore_CNN...`），
/// 见 Anime4K v4 官方 mpv 配方的推荐顺序。
#[derive(Clone, Debug, PartialEq)]
pub struct Anime4kConfig {
    /// GLSL shader 文件，依行使顺序列出。
    pub shader_files: Vec<PathBuf>,
    /// 输出缩放上限（Anime4K 指令如 2x/4x；引擎据此约束输出分辨率）。
    pub target_scale: f32,
    /// 显示层忽略目标分辨率而依原分辨率放置（mpv 的 `--glsl-shader` 行为参照）。
    /// Anime4K 网络渲染尺寸大于视频原尺寸时，显示层仍按原视频分辨率。
    pub display_size_from_source: bool,
}

impl Anime4kConfig {
    /// 常规起步：Restore_CNN_M + Upscale_CNN_x2_M（官方推荐中档组合，
    /// 文件路径由运行侧提供，本函数只固定次序名）。
    pub fn typical_order(names: impl IntoIterator<Item = PathBuf>) -> Self {
        Self {
            shader_files: names.into_iter().collect(),
            target_scale: 2.0,
            display_size_from_source: true,
        }
    }
}

/// 校验占位：引擎实现各自执行；本 crate 保证配置形态可用（shader 列表非空等）。
pub fn validate(policy: &UpscalePolicy) -> Result<()> {
    match policy {
        UpscalePolicy::None => Ok(()),
        UpscalePolicy::Anime4k(cfg) if cfg.shader_files.is_empty() => {
            Err(crate::error::Error::Engine(
                "Anime4k 策略必须有至少一个 shader 文件".into(),
            ))
        }
        UpscalePolicy::Anime4k(_) => Ok(()),
    }
}
