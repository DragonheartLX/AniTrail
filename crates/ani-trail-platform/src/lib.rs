//! 番迹 · 平台化差异统一接口（docs/规划/01 §5 / 06b D37）。
//!
//! 职责：把「桌面 dirs 体系 vs Android 应用内部存储」的落盘收敛为**单一根目录 `ani-trail/`**
//! + 根下分类子目录（docs/规划/01 §5 / 06b D37）：
//!
//! ```text
//! <root>/                       # 桌面 = data_local_dir()/ani-trail；Android = FilesDir()/ani-trail
//! ├─ config/config.toml         # 设置全量（唯一进 TOML 的文件）
//! ├─ db/anitrail.db             # SQLite（WAL 同目录）
//! ├─ cache/images/              # 海报/头像磁盘缓存（sha256 命名，LRU）
//! └─ export/                    # 年鉴分享图等着眼导出的产物
//! ```
//!
//! 许可：只依赖纯 Rust 工具（anyhow/dirs），无 slint/tela-ui/网络栈，任何 crate 都可引。
//!
//! Android 沙箱无 `$HOME`/XDG，`dirs` 恒返 None——未注入 FilesDir 前 [`layout`] 返回
//! Err（调用方按“默认链内存配置”降级，见 `ani-trail` bin main）。

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// 布局根：宿主默认（dirs 体系）或注入的内部存储根（Android FilesDir / 测试）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformBase {
    /// 桌面：根目录 = `dirs::data_local_dir()/ani-trail`。
    Host,
    /// 注入内部存储根（Android FilesDir / 测试临时目录）。
    Files(PathBuf),
}

/// 统一落盘布局：根目录下分类保存。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageLayout {
    /// 存储根目录（一切产品物的父目录 `ani-trail/`）。
    pub root: PathBuf,
    /// 设置全量：`config/config.toml`。
    pub config_path: PathBuf,
    /// SQLite 数据库：`db/anitrail.db`（WAL 文件同目录）。
    pub db_path: PathBuf,
    /// 海报/头像磁盘缓存：`cache/images/`（sha256 命名，LRU 上限 M8 定值）。
    pub images_dir: PathBuf,
    /// 导出产物：`export/`（年鉴分享图等着眼落地的类别目录）。
    pub export_dir: PathBuf,
}

/// 平台名（日志/关于页用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    Mac,
    Android,
}

/// 单根 + 分类（纯函数，无 env 依赖；调用方保证目录懒创建由落盘方负责）。
pub fn layout_from_root(root: &Path) -> StorageLayout {
    StorageLayout {
        root: root.to_owned(),
        config_path: root.join("config").join("config.toml"),
        db_path: root.join("db").join("anitrail.db"),
        images_dir: root.join("cache").join("images"),
        export_dir: root.join("export"),
    }
}

/// 平台默认布局。Android 未注入 FilesDir 前是 Err（M9-3 接 `ndk_context`）。
pub fn layout(base: PlatformBase) -> Result<StorageLayout> {
    let root = match base {
        PlatformBase::Host => match platform() {
            Platform::Android => {
                anyhow::bail!("Android FilesDir 未注入（M9-3：ndk_context → PlatformBase::Files）")
            }
            _ => dirs::data_local_dir()
                .context("系统数据目录不可得（HOME/XDG 异常）")?
                .join("ani-trail"),
        },
        PlatformBase::Files(base) => base.join("ani-trail"),
    };
    Ok(layout_from_root(&root))
}

/// 当前编译平台（cfg 静态）。
pub fn platform() -> Platform {
    if cfg!(target_os = "android") {
        Platform::Android
    } else if cfg!(target_os = "macos") {
        Platform::Mac
    } else if cfg!(windows) {
        Platform::Windows
    } else {
        Platform::Linux
    }
}

mod token;
pub use token::TokenStore;

#[cfg(test)]
mod tests {
    use super::*;

    /// 注入单根：根目录分类子目录（Android FilesDir 口径）。
    #[test]
    fn injected_root_categorised() -> anyhow::Result<()> {
        let tmp = tempfile::tempdir()?;
        let layout = layout(PlatformBase::Files(tmp.path().to_path_buf()))?;
        let base = tmp.path().join("ani-trail");
        assert_eq!(layout.root, base);
        assert_eq!(layout.config_path, base.join("config").join("config.toml"));
        assert_eq!(layout.db_path, base.join("db").join("anitrail.db"));
        assert_eq!(layout.images_dir, base.join("cache").join("images"));
        assert_eq!(layout.export_dir, base.join("export"));
        // 分类都在根内（不外溢）。
        for p in [&layout.config_path, &layout.db_path] {
            assert!(p.starts_with(&base));
        }
        Ok(())
    }

    /// 单根纯函数：根下 config/db/cache/export 四类（docs/规划/01 §5）。
    #[test]
    fn single_root_layout_categories() {
        let l = layout_from_root(Path::new("/var/data/ani-trail"));
        assert_eq!(l.root, PathBuf::from("/var/data/ani-trail"));
        assert_eq!(
            l.config_path,
            PathBuf::from("/var/data/ani-trail/config/config.toml")
        );
        assert_eq!(
            l.db_path,
            PathBuf::from("/var/data/ani-trail/db/anitrail.db")
        );
        assert_eq!(
            l.images_dir,
            PathBuf::from("/var/data/ani-trail/cache/images")
        );
        assert_eq!(l.export_dir, PathBuf::from("/var/data/ani-trail/export"));
    }
}
