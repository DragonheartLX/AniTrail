//! 内置/参考 Provider 形状与通用工具。
//!
//! M6-6 起落地的 kazumi 规则 Provider（Xpath+Css 双方言引擎）与 P1 的 BT Provider
//! 均实现 [`crate::provider::SourceProvider`]。本模块先冻结两个公共件：
//! * [`StaticProvider`] —— 纯内存条目的参考实现（测试、无网络演示、本地缓存列表直连引擎）；
//! * [`dialect_miss`] —— 上层把"规则方言不受支持"标准化成 [`crate::Error`]。

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::error::{Error, Result};
use crate::{EpisodeSource, SearchEntry, SelectorDialect, SourceProvider};

/// 纯内存 Provider：`ref_id → 网直链块`。
///
/// 用途（测试 fixture / 断网演示 / 本地缓存直接出集）。生产 Provider 实现可借用其
/// "抽象引用 → 具体源" 的形态：上层永远只见 [`SearchEntry::ref_id`]。
#[derive(Debug)]
pub struct StaticProvider {
    name: String,
    dialect: SelectorDialect,
    catalog: Mutex<BTreeMap<String, Vec<EpisodeSource>>>,
}

impl StaticProvider {
    pub fn new(name: &str, dialect: SelectorDialect) -> Self {
        Self {
            name: name.to_owned(),
            dialect,
            catalog: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn put(&self, ref_id: &str, items: Vec<EpisodeSource>) {
        self.catalog
            .lock()
            .unwrap()
            .insert(ref_id.to_owned(), items);
    }

    fn pull(&self, ref_id: &str) -> Result<Vec<EpisodeSource>> {
        self.locked_cat(ref_id)
            .ok_or_else(|| Error::InvalidRef(ref_id.to_owned()))
    }

    fn locked_cat(&self, ref_id: &str) -> Option<Vec<EpisodeSource>> {
        self.catalog.lock().unwrap().get(ref_id).cloned()
    }
}

#[async_trait::async_trait]
impl SourceProvider for StaticProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn dialect(&self) -> SelectorDialect {
        self.dialect
    }

    async fn search(&self, keyword: &str) -> Result<Vec<SearchEntry>> {
        let keys = {
            let guard = self.catalog.lock().unwrap();
            guard
                .keys()
                .filter(|k| k.contains(keyword))
                .cloned()
                .collect::<Vec<_>>()
        };
        Ok(keys
            .into_iter()
            .map(|ref_id| SearchEntry {
                title: ref_id.clone(),
                ref_id,
            })
            .collect())
    }

    async fn episodes(&self, ref_id: &str) -> Result<Vec<EpisodeSource>> {
        self.pull(ref_id)
    }
}

/// 规则方言不受支持时的标准错误。
pub fn dialect_miss(provider: &str, found: SelectorDialect, allowed: &[SelectorDialect]) -> Error {
    Error::Upstream {
        provider: provider.to_owned(),
        detail: format!(
            "规则方言 {found:?} 不在允许列表 {allowed:?}（Provider 版本过旧或规则非法）"
        ),
    }
}
