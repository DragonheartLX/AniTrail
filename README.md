# AniTrail · 番迹

Rust workspace：集成 Bangumi 记录与多源播放的番剧记录应用。

## 布局

```
crates/
  ani-trail-platform/  # 平台化差异统一接口（存储布局/令牌契约，D37）
  ani-trail-bangumi/   # Bangumi 对接：client·mirror·tls(ECH)·auth·rate_limit
  ani-trail-player/    # 播放引擎契约（mpv Render API 方向，D3）
  ani-trail-source/    # 片源 Provider 契约（kazumi-rules XPath/CSS 双方言，D31/D33）
  ani-trail-core/      # 核心库（media 桥；DB/sync 归 M4 落地）
  ani-trail/           # 主程序 bin：ui + 路由/壳 + 配置/日志 + splash 驱动
```

## 构建

```
cargo build
cargo run -p ani-trail       # 启动主窗口（无配置自动落默认镜像链）
cargo test --workspace       # 全量单测
cargo clippy --workspace --all-targets -- -D warnings
```

## UI 组件预览 / 交互验收

```
# 组件 gallery（M3-6）：六件组件 showcase
slint-viewer --check -L tela-ui=$(ls -d ~/.cargo/registry/src/*/tela-ui-0.1.1/tela-ui) crates/ani-trail/ui/dev/gallery.slint
SLINT_BACKEND=headless slint-viewer --style fluent -L tela-ui=… \
  --screenshot /tmp/gallery.png crates/ani-trail/ui/dev/gallery.slint

# 交互验收驱动（需应用常驻出 MCP 端点）：
SLINT_BACKEND=headless SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=9430 \
  timeout 0 cargo run -q -p ani-trail --features slint/mcp
python3 scripts/mcp-nav-check.py http://127.0.0.1:9430/mcp   # M2 导航/返回/深度
python3 scripts/mcp-d2-check.py    http://127.0.0.1:9430/mcp   # D2 下钻/滚动回填（像素 diff）
```

## 配置

- 位置：`config_local_dir()/ani-trail/config/config.toml`（单根布局见 `ani-trail-platform`）
- 无文件 = 默认镜像链 `api.bgmapi.com → api.bgm.tv`；`[[mirrors]] label/api_base/ech_config_b64`
- 令牌不入明文配置（keyring / EncryptedSharedPreferences 契约 in platform）

## License

Apache-2.0（另有 tela-ui 0.1.1 / Lucide（ISC）致谢见规划文档）
