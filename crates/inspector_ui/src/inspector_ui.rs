// ══════════════════════════════════════════════════════════════════════════════
// RLT 分支改动（保留项）—— 唯一目的：让 release 构建也能启用元素检查器
//
// 【改了什么】下面 4 处门控从 `#[cfg(debug_assertions)]` 放宽为
//   `#[cfg(any(feature = "inspector", debug_assertions))]`（含其 `not(...)` 反相）。
//
// 【为什么改】rlt-dev 自研的元素检查器（`应用/源码/子系统/元素检查器.rs`，
//   需求 FR-APP-022）复用本 crate 的**拾取 / 高亮 / DivInspectorState 回读**机制。
//   注意：只复用机制，**不复制本 crate 的代码** —— 本 crate 是 GPL-3.0-or-later，
//   而 rlt-dev 只依赖 Apache-2.0 的 gpui 家族，实现全部自写。
//   而上游把整块 inspector 锁死在 `#[cfg(debug_assertions)]` ⇒ rlt-dev 的发布档
//   （release）拿不到检查器，故必须开一个显式开关。
//
// 【为什么不影响上游】默认不开启 `inspector` feature ⇒ `any(...)` 退化为原
//   `debug_assertions`，编译产物与上游**逐字节一致**（release 仍走本文件末尾的兜底
//   `init`）。要启用必须显式 `--features inspector`。
//
// 【性质】这是 rlt-dev 对 fork 的**必需改动**，不是上游缺陷修复 ⇒ 不向上游回贡。
//
// 【关联】`crates/zed/Cargo.toml`、`crates/ui/Cargo.toml` 的同名 feature，
//   以及 `crates/ui/src/traits/styled_ext.rs` 的反射模块门控（须与本处同步）。
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(any(feature = "inspector", debug_assertions))]
mod div_inspector;
#[cfg(any(feature = "inspector", debug_assertions))]
mod inspector;

#[cfg(any(feature = "inspector", debug_assertions))]
pub use inspector::init;

#[cfg(not(any(feature = "inspector", debug_assertions)))]
pub fn init(_app_state: std::sync::Arc<workspace::AppState>, cx: &mut gpui::App) {
    use std::any::TypeId;
    use workspace::notifications::NotifyResultExt as _;

    cx.on_action(|_: &zed_actions::dev::ToggleInspector, cx| {
        Err::<(), anyhow::Error>(anyhow::anyhow!(
            "dev::ToggleInspector is only available in debug builds"
        ))
        .notify_app_err(cx);
    });

    command_palette_hooks::CommandPaletteFilter::update_global(cx, |filter, _cx| {
        filter.hide_action_types(&[TypeId::of::<zed_actions::dev::ToggleInspector>()]);
    });
}
