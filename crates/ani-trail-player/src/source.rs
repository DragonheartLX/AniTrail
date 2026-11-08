//! 播放的媒体源与剧集队列。
//!
//! 来源：本地文件路径或 HTTP(S) URL（Bangumi 外部资源站 / 用户本地文件）。
//! 连播语义属于 MVP 需求（自动下一集），队列放本 crate 统一，UI 与引擎都可见。

/// 一个可播放条目。`uri` 同时覆盖本地路径与 `http(s)://` URL。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub uri: String,
    pub title: Option<String>,
}

impl Source {
    /// 本地文件。
    pub fn local<P: Into<String>>(path: P) -> Self {
        Self {
            uri: path.into(),
            title: None,
        }
    }

    /// 远端 URL。
    pub fn remote<U: Into<String>>(url: U) -> Self {
        Self {
            uri: url.into(),
            title: None,
        }
    }

    pub fn with_title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
}

/// 剧集播放队列。`cursor` 指向当前条目；`None` = 未开始播放。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Playlist {
    items: Vec<Source>,
    cursor: Option<usize>,
}

impl Playlist {
    pub fn new() -> Self {
        Self::default()
    }

    /// 从条目列表构造，游标默认第 0 项。
    pub fn from_sources<I: IntoIterator<Item = Source>>(iter: I) -> Self {
        let items: Vec<Source> = iter.into_iter().collect();
        let cursor = if items.is_empty() { None } else { Some(0) };
        Self { items, cursor }
    }

    pub fn append(&mut self, src: Source) {
        self.items.push(src);
        if self.cursor.is_none() {
            self.cursor = Some(0);
        }
    }

    pub fn current(&self) -> Option<&Source> {
        self.cursor.and_then(|i| self.items.get(i))
    }

    /// 跳到指定下标；越界不动游标并返回 None。
    pub fn set_cursor(&mut self, idx: usize) -> Option<&Source> {
        if idx < self.items.len() {
            self.cursor = Some(idx);
        }
        self.current_matching(idx)
    }

    /// 仅当下标合法时返回该项。
    fn current_matching(&self, idx: usize) -> Option<&Source> {
        self.items.get(idx)
    }

    /// 前进一集；已在最后一集则停留并返回 None。
    pub fn advance(&mut self) -> Option<&Source> {
        let c = self.cursor?;
        if c + 1 < self.items.len() {
            self.cursor = Some(c + 1);
            self.current()
        } else {
            None
        }
    }

    /// 后退一集；已在第一集则返回 None。
    pub fn prev(&mut self) -> Option<&Source> {
        if let Some(c) = self.cursor {
            if c > 0 {
                self.cursor = Some(c - 1);
            }
        }
        self.current()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playlist_cursor_walk_bounds() {
        let mut pl = Playlist::from_sources([
            Source::local("a"),
            Source::local("b"),
            Source::remote("https://x"),
        ]);
        assert_eq!(pl.current().map(|s| s.uri.as_str()), Some("a"));
        assert_eq!(pl.advance().map(|s| s.uri.as_str()), Some("b"));
        assert_eq!(pl.advance().map(|s| s.uri.as_str()), Some("https://x"));
        assert_eq!(pl.advance().map(|s| s.uri.as_str()), None, "末尾停留");
        assert_eq!(pl.prev().map(|s| s.uri.as_str()), Some("b"));
        assert!(pl.set_cursor(9).is_none());
        assert_eq!(pl.current().map(|s| s.uri.as_str()), Some("b"));
    }

    #[test]
    fn playlist_empty_then_start_zero() {
        let mut pl = Playlist::new();
        assert!(pl.is_empty());
        assert_eq!(pl.current(), None);
        pl.append(Source::local("a").with_title("eps1"));
        assert_eq!(pl.len(), 1);
        assert_eq!(pl.current().map(|s| s.title.as_deref()), Some(Some("eps1")));
    }
}
