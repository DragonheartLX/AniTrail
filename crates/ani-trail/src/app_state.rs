//! 应用状态单例（docs/规划/01 §3）：登录态、mirror 配置、bridge 类型只传值。
//!
//! 状态全部驻 UI（事件循环＝主）线程：slint 属性限于 UI 线程；后台驱动（如
//! splash.rs）经 `invoke_from_event_loop` 进本线程访问——统一构造成本最小。
//! M2 持路由栈并负责把回填写进壳层；M4 起接入 sync/DB。

use std::cell::RefCell;

use slint::{ComponentHandle as _, Weak};

use crate::router::RouterState;
use crate::{MainWindow, Router};

pub struct AppState {
    router: RouterState,
}

impl AppState {
    fn new(window: Weak<MainWindow>) -> Self {
        let emit: crate::router::RouterEmit = std::rc::Rc::new(move |route, restored, depth| {
            if let Some(win) = window.upgrade() {
                let router = win.global::<Router<'_>>();
                router.set_current(route);
                router.set_depth(depth);
                router.set_restored_scroll(restored as f32);
            }
        });
        Self {
            router: RouterState::new(emit),
        }
    }

    /// 路由栈（UI 线程内可变访问：go/back/save-scroll/boot）。
    pub fn router(&mut self) -> &mut RouterState {
        &mut self.router
    }
}

thread_local! {
    // None = 未初始化；Some = 已 init（仅 main() 一次）。
    static STATE: RefCell<Option<AppState>> = const { RefCell::new(None) };
}

/// main() 首行调用；保 thread_local 单例。
pub fn init_state(window: Weak<MainWindow>) {
    STATE.with_borrow_mut(|slot| {
        assert!(slot.is_none(), "AppState 只能 init 一次");
        *slot = Some(AppState::new(window));
    });
}

/// 事件循环线程内访问应用状态（UI 回调/后台线程投递的闭包均在此线程跑到）。
pub fn with_state<R>(f: impl FnOnce(&mut AppState) -> R) -> R {
    STATE.with_borrow_mut(|slot| {
        f(slot
            .as_mut()
            .expect("AppState 未初始化：先在 main 调 init_state"))
    })
}
