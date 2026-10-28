//! A frame-wide, read-only **paint observation hook**.
//!
//! ## Why this exists
//!
//! GPUI's element tree is immediate-mode: every frame rebuilds it, and there is no
//! retained element tree to walk afterwards. Downstream tools that need to enumerate
//! **every element that a frame actually paints** (e.g. RLT's element inspector
//! layout-tree view, which used to hand-wrap ~5 regions with audit wrappers) therefore
//! had no generic entry point. This module adds the smallest possible one.
//!
//! ## Contract
//!
//! * **Zero cost when unused**: one relaxed atomic load per painted element.
//! * **Read-only**: the hook runs *around* `Element::paint`; it cannot affect layout,
//!   painting, hitboxes or accessibility.
//! * **Nesting order**: `Enter` is emitted before an element's `paint`, and `Exit`
//!   after it returns, so consumers can rebuild the exact nesting tree by stack.
//! * **Deferred elements** (`gpui::deferred`) are painted outside their ancestor's
//!   `paint` call; consumers see them at root level and must reconcile with
//!   [`PaintEvent::global_id`] if they need exact parentage.
//!
//! ## Example (RLT side)
//!
//! ```ignore
//! gpui::set_paint_observer(Some(my_observer));
//! // ... draw one frame ...
//! gpui::set_paint_observer(None);
//! ```

use std::panic::Location;
use std::sync::RwLock;

use crate::{Bounds, Pixels};

/// One painted element, reported to the observer.
pub struct PaintEvent<'a> {
    /// Concrete Rust type name of the element (`std::any::type_name::<E>()`).
    pub element_type: &'static str,
    /// Where the element value was constructed, when the element tracks it.
    pub source: Option<&'static Location<'static>>,
    /// Ancestor `ElementId` path at paint time (empty when no ancestor has an id).
    pub global_id: Option<&'a crate::GlobalElementId>,
    /// Absolute paint bounds in window coordinates.
    pub bounds: Bounds<Pixels>,
}

/// What happened to the observed element.
pub enum PaintObservation<'a> {
    /// The element is about to paint; its children (if any) are painted before `Exit`.
    Enter(PaintEvent<'a>),
    /// The element and all of its children finished painting.
    Exit,
}

/// Signature of the observer callback.
pub type PaintObserver = fn(PaintObservation<'_>);

static OBSERVER: RwLock<Option<PaintObserver>> = RwLock::new(None);

/// Installs (or, with `None`, removes) the frame-wide paint observer.
///
/// The observer is invoked on whichever thread paints the frame; it is expected to be
/// installed and removed from the same thread (RLT installs it around one collection
/// frame on the main thread).
pub fn set_paint_observer(observer: Option<PaintObserver>) {
    if let Ok(mut slot) = OBSERVER.write() {
        *slot = observer;
    }
}

/// Whether an observer is currently installed.
pub fn has_paint_observer() -> bool {
    OBSERVER.read().map(|slot| slot.is_some()).unwrap_or(false)
}

fn observer() -> Option<PaintObserver> {
    OBSERVER.read().ok().and_then(|slot| *slot)
}

#[inline]
pub(crate) fn notify_enter(event: PaintEvent<'_>) {
    if let Some(observer) = observer() {
        observer(PaintObservation::Enter(event));
    }
}

#[inline]
pub(crate) fn notify_exit() {
    if let Some(observer) = observer() {
        observer(PaintObservation::Exit);
    }
}
