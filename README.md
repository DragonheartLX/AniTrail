# 番迹 AniTrail

Rust workspace，按功能拆分 crates。

## 布局

```
crates/
  ani-trail-platform/   # 平台存储/令牌统一接口（D37）
  ani-trail-core/       # 核心库（media 桥）
  ani-trail-bangumi/    # Bangumi 对接（client·mirror·tls·auth）
  ani-trail-player/     # 播放引擎契约
  ani-trail-source/     # 片源 Provider 契约（D33）
  ani-trail/            # 主程序入口 (bin: ani-trail)
```

## 构建

```
cargo build
cargo run -p ani-trail
```

## License

Apache-2.0
