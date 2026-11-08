//! 契约测试（provider 形状 / 方言 / 错误）。

use std::sync::Arc;

use crate::error::Error;
use crate::provider::{dialect_miss, StaticProvider};
use crate::{EpisodeSource, SelectorDialect, SourceProvider};

// 动态分发可用性验证（Provider 列表在 UI 侧就是 Box<dyn SourceProvider>）。
type Providers = Vec<Arc<dyn SourceProvider>>;

#[tokio::test]
async fn static_provider_search_and_pick() {
    let p = StaticProvider::new("本地演示", SelectorDialect::Xpath);
    p.put(
        "某番",
        vec![
            EpisodeSource {
                index: 1,
                title: Some("EP1".into()),
                uri: "https://cdn/1.m3u8".into(),
            },
            EpisodeSource {
                index: 2,
                title: None,
                uri: "https://cdn/2.m3u8".into(),
            },
        ],
    );
    let providers: Providers = vec![Arc::new(p)];
    let results = providers[0].search("某").await.unwrap();
    assert_eq!(results[0].ref_id, "某番");

    let eps = providers[0].episodes("某番").await.unwrap();
    assert_eq!(eps.len(), 2);
    assert_eq!(eps[0].index, 1);

    assert!(
        matches!(
            providers[0].episodes("unknown").await,
            Err(Error::InvalidRef(_))
        ),
        "未知 ref_id → InvalidRef"
    );
}

#[test]
fn dialect_miss_maps_to_error() {
    let err = dialect_miss("demo", SelectorDialect::Css, &[SelectorDialect::Xpath]);
    assert!(matches!(err, Error::Upstream { .. }), "{err}");
}
