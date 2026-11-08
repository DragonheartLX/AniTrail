//! M2-1 · 路由栈（docs/规划/01 §3）：Rust 持 `Vec<Route>`，页面懒挂载分发在 Slint 壳。
//!
//! 规则（D2 验收）：
//! - `go(route)`：目标 == 当前忽略；其余 push；
//! - `back()`：弹栈并回填目标页滚动；栈底（splash）忽略；
//! - 滚动持久化：`scroll: Vec<(Route, f64)>`，push/back 都经 `apply_current` 回填；
//! - Route 枚举由 slint-build 生成（ui/router.slint 的 `Route`，Splash 为 0 号）。
//!
//! UI 回填走 [`RouterEmit`]：窗口耦合由 `main.rs` 注入，测试注入捕获器。

use tracing::info;

use crate::Route;

pub type RouterEmit = std::rc::Rc<dyn Fn(Route, f64, i32)>;

pub struct RouterState {
    stack: Vec<Route>,
    scroll: Vec<(Route, f64)>,
    emit: RouterEmit,
}

impl RouterState {
    pub fn new(emit: RouterEmit) -> Self {
        Self {
            stack: vec![Route::Splash],
            scroll: Vec::new(),
            emit,
        }
    }

    /// 深层进入（push + 回填）。目标与当前相同则忽略。
    pub fn go(&mut self, route: Route) {
        info!(target: "ani_trail", "router::go {:?}", route);
        if self.stack.last() == Some(&route) {
            return;
        }
        self.stack.push(route);
        self.apply_current(route);
    }

    /// 返回（pop + 回填）。栈底忽略。
    pub fn back(&mut self) {
        info!(target: "ani_trail", "router::back depth {}", self.stack.len());
        if self.stack.len() <= 1 {
            return;
        }
        self.stack.pop();
        if let Some(prev) = self.stack.last().copied() {
            self.apply_current(prev);
        }
    }

    /// 主导航节切换：清栈到根、单条落位（无返回钮语境；D40）。
    pub fn switch(&mut self, route: Route) {
        info!(target: "ani_trail", "router::switch {:?}", route);
        if self.stack.len() == 1 && self.stack.last() == Some(&route) {
            return;
        }
        self.stack.clear();
        self.stack.push(route);
        self.apply_current(route);
    }

    /// 页面滚动上报（scaffold 持续调用）。
    pub fn save_scroll(&mut self, y: f64) {
        if let Some(top) = self.stack.last().copied() {
            if let Some(slot) = self.scroll.iter_mut().find(|(r, _)| *r == top) {
                slot.1 = y;
            } else {
                self.scroll.push((top, y));
            }
        }
    }

    fn apply_current(&mut self, route: Route) {
        let restored = self
            .scroll
            .iter()
            .find(|(r, _)| *r == route)
            .map(|(_, y)| *y)
            .unwrap_or(0.0);
        (self.emit)(route, restored, self.stack.len() as i32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// M2-1/2 语义：push/back 栈序正确；回填取「该页」保存的滚动偏移，未保存页回 0。
    #[test]
    fn router_go_back_and_scroll_memory() {
        let routed: std::rc::Rc<std::cell::RefCell<Vec<(Route, f64, i32)>>> = Default::default();
        let routed_c = std::rc::Rc::clone(&routed);
        let state = std::rc::Rc::new(std::cell::RefCell::new(RouterState::new(std::rc::Rc::new(
            move |route, restored, depth| {
                routed_c.borrow_mut().push((route, restored, depth));
            },
        ))));

        state.borrow_mut().go(Route::Home);
        state.borrow_mut().save_scroll(120.0);
        state.borrow_mut().go(Route::Progress);
        state.borrow_mut().go(Route::Settings);
        {
            let s = state.borrow();
            assert_eq!(
                s.stack,
                vec![Route::Splash, Route::Home, Route::Progress, Route::Settings]
            );
            assert_eq!(s.scroll[0].1, 120.0);
        }

        state.borrow_mut().go(Route::Settings); // 平等目标忽略
        assert_eq!(state.borrow().stack.len(), 4);

        state.borrow_mut().back(); // → Progress
        state.borrow_mut().back(); // → Home（回填 120, depth 2）
        assert_eq!(
            routed.borrow().last().copied(),
            Some((Route::Home, 120.0, 2))
        );

        state.borrow_mut().back(); // → Splash
        state.borrow_mut().back(); // 栈底忽略
        assert_eq!(state.borrow().stack, vec![Route::Splash]);
        assert_eq!(
            routed.borrow().last().copied(),
            Some((Route::Splash, 0.0, 1)),
            "splash 未存滚动 → 回填 0"
        );
    }
}
