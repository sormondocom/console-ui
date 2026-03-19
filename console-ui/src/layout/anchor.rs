//! # Anchor Layout
//!
//! Relative, daisy-chainable positioning for console widgets — inspired by
//! Java Swing's `SpringLayout` / anchor constraints, adapted for a terminal
//! cell grid.
//!
//! Instead of specifying absolute coordinates, you declare **relationships**
//! between edges of widgets and let the layout engine resolve them into
//! concrete `Rect` positions before rendering.
//!
//! ## Concepts
//!
//! - **`AnchorId`** — a stable handle for a widget slot in the layout.
//!   A special built-in id `AnchorId::CONTAINER` refers to the bounding box
//!   of the whole layout region.
//!
//! - **`Edge`** — one of the four sides of a widget: `Top`, `Right`,
//!   `Bottom`, `Left`.
//!
//! - **`Constraint`** — "edge A of widget X is `offset` cells away from
//!   edge B of widget Y".  Offsets are signed: positive pushes the edge
//!   further in the natural direction (e.g. positive offset on a `Left`
//!   constraint moves the widget right).
//!
//! - **`AnchorLayout`** — the layout manager itself.  Add widgets with
//!   `add()`, attach constraints with `constrain()`, then call `render()`
//!   or `resolve()` to compute positions.
//!
//! ## Example
//!
//! ```rust,no_run
//! use console_ui::layout::anchor::{AnchorLayout, Edge};
//! use console_ui::widget::Panel;
//! use console_ui::canvas::Canvas;
//! use console_ui::prelude::*;
//!
//! let mut layout = AnchorLayout::new(80, 24);
//!
//! let title = layout.add(
//!     Box::new(Panel::new().title("Title")),
//!     Some((40, 3)),   // preferred size hint
//! );
//!
//! let content = layout.add(
//!     Box::new(Panel::new().title("Content")),
//!     None,
//! );
//!
//! // Centre the title horizontally inside the container.
//! layout.constrain_centre_h(title);
//!
//! // Pin title to the top of the container with a 1-cell margin.
//! layout.constrain(title, Edge::Top, AnchorLayout::CONTAINER, Edge::Top, 1);
//!
//! // Place content directly below the title, also left-aligned with it.
//! layout.constrain(content, Edge::Top,  title,                Edge::Bottom, 1);
//! layout.constrain(content, Edge::Left, title,                Edge::Left,   0);
//! layout.constrain(content, Edge::Right, AnchorLayout::CONTAINER, Edge::Right, -1);
//! layout.constrain(content, Edge::Bottom, AnchorLayout::CONTAINER, Edge::Bottom, -1);
//!
//! let mut canvas = Canvas::new(80, 24);
//! let mut root = canvas.sub(0, 0, 80, 24);
//! layout.render(&mut root);
//! ```

use std::collections::HashMap;
use crate::canvas::SubCanvas;
use crate::widget::traits::Widget;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A stable identifier for a widget slot within an `AnchorLayout`.
/// `AnchorLayout::CONTAINER` is the reserved ID for the layout's bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnchorId(u32);

/// Which edge of a widget a constraint refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Top,
    Right,
    Bottom,
    Left,
}

/// A single constraint: widget `src`'s `src_edge` should be `offset` cells
/// from widget `dst`'s `dst_edge`.
///
/// Positive offsets:
/// - `Top`:    pushes the top edge down (increases y)
/// - `Left`:   pushes the left edge right (increases x)
/// - `Bottom`: pushes the bottom edge up (decreases effective height)
/// - `Right`:  pushes the right edge left (decreases effective width)
#[derive(Debug, Clone, Copy)]
pub struct Constraint {
    pub src:      AnchorId,
    pub src_edge: Edge,
    pub dst:      AnchorId,
    pub dst_edge: Edge,
    pub offset:   i16,
}

// ---------------------------------------------------------------------------
// Internal resolved rect state
// ---------------------------------------------------------------------------

/// A partially-resolved position.  `None` means not yet determined.
#[derive(Debug, Default, Clone, Copy)]
struct Resolved {
    top:    Option<i16>,
    left:   Option<i16>,
    bottom: Option<i16>,
    right:  Option<i16>,
}

impl Resolved {
    fn width(&self) -> Option<u16> {
        match (self.left, self.right) {
            (Some(l), Some(r)) if r > l => Some((r - l) as u16),
            _ => None,
        }
    }
    fn height(&self) -> Option<u16> {
        match (self.top, self.bottom) {
            (Some(t), Some(b)) if b > t => Some((b - t) as u16),
            _ => None,
        }
    }
    fn to_rect(&self, pref_w: u16, pref_h: u16) -> crate::layout::Rect {
        let l = self.left.unwrap_or(0).max(0) as u16;
        let t = self.top.unwrap_or(0).max(0) as u16;
        let w = self.width().unwrap_or(pref_w);
        let h = self.height().unwrap_or(pref_h);
        crate::layout::Rect::new(l, t, w, h)
    }
}

// ---------------------------------------------------------------------------
// AnchorLayout
// ---------------------------------------------------------------------------

struct Slot {
    widget:   Box<dyn Widget>,
    pref_w:   u16,
    pref_h:   u16,
}

/// Relative-anchor layout manager.
///
/// Widgets are positioned by declaring edge-to-edge constraints.  The
/// resolver performs up to `MAX_PASSES` forward passes over all constraints
/// until every widget has a fully-determined position (or the pass limit is
/// reached, at which point widgets fall back to their preferred sizes at
/// position `(0,0)`).
pub struct AnchorLayout {
    width:       u16,
    height:      u16,
    next_id:     u32,
    slots:       HashMap<AnchorId, Slot>,
    constraints: Vec<Constraint>,
    /// Resolved positions — populated by `resolve()`.
    resolved:    HashMap<AnchorId, Resolved>,
}

impl AnchorLayout {
    /// The reserved `AnchorId` representing the layout's bounding box.
    pub const CONTAINER: AnchorId = AnchorId(0);

    const MAX_PASSES: usize = 32;

    /// Create a new layout that will render into a region of `width × height`.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            next_id:     1,
            slots:       HashMap::new(),
            constraints: Vec::new(),
            resolved:    HashMap::new(),
        }
    }

    /// Update the layout bounds (e.g. after a terminal resize).
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width  = width;
        self.height = height;
        self.resolved.clear();
    }

    // ------------------------------------------------------------------
    // Adding widgets
    // ------------------------------------------------------------------

    /// Add a widget and return its `AnchorId`.
    ///
    /// `size_hint` overrides the widget's `preferred_size()` if provided.
    pub fn add(&mut self, widget: Box<dyn Widget>, size_hint: Option<(u16, u16)>) -> AnchorId {
        let (pref_w, pref_h) = size_hint.unwrap_or_else(|| widget.preferred_size());
        let id = AnchorId(self.next_id);
        self.next_id += 1;
        self.slots.insert(id, Slot { widget, pref_w, pref_h });
        self.resolved.clear();
        id
    }

    // ------------------------------------------------------------------
    // Adding constraints
    // ------------------------------------------------------------------

    /// Add a constraint: `src_edge` of `src` is `offset` cells from `dst_edge` of `dst`.
    ///
    /// Constraints form a DAG.  Cycles are silently broken on the last pass.
    pub fn constrain(
        &mut self,
        src: AnchorId, src_edge: Edge,
        dst: AnchorId, dst_edge: Edge,
        offset: i16,
    ) {
        self.constraints.push(Constraint { src, src_edge, dst, dst_edge, offset });
        self.resolved.clear();
    }

    /// Convenience: pin all four edges of `id` to the container with the
    /// given margin on each side.
    pub fn fill(&mut self, id: AnchorId, margin: i16) {
        let c = Self::CONTAINER;
        self.constrain(id, Edge::Top,    c, Edge::Top,     margin);
        self.constrain(id, Edge::Left,   c, Edge::Left,    margin);
        self.constrain(id, Edge::Bottom, c, Edge::Bottom, -margin);
        self.constrain(id, Edge::Right,  c, Edge::Right,  -margin);
    }

    /// Convenience: centre `id` horizontally within the container,
    /// using the widget's preferred width.
    pub fn constrain_centre_h(&mut self, id: AnchorId) {
        let pref_w = self.slots.get(&id).map(|s| s.pref_w).unwrap_or(20);
        let left   = ((self.width as i16) - pref_w as i16) / 2;
        self.constrain(id, Edge::Left,  Self::CONTAINER, Edge::Left, left);
        self.constrain(id, Edge::Right, Self::CONTAINER, Edge::Left, left + pref_w as i16);
    }

    /// Convenience: centre `id` vertically within the container.
    pub fn constrain_centre_v(&mut self, id: AnchorId) {
        let pref_h = self.slots.get(&id).map(|s| s.pref_h).unwrap_or(10);
        let top    = ((self.height as i16) - pref_h as i16) / 2;
        self.constrain(id, Edge::Top,    Self::CONTAINER, Edge::Top, top);
        self.constrain(id, Edge::Bottom, Self::CONTAINER, Edge::Top, top + pref_h as i16);
    }

    /// Convenience: pin `src` immediately after `dst` (below it) with an
    /// optional gap, and left-align them.
    pub fn below(&mut self, src: AnchorId, dst: AnchorId, gap: i16) {
        self.constrain(src, Edge::Top,  dst, Edge::Bottom, gap);
        self.constrain(src, Edge::Left, dst, Edge::Left,   0);
    }

    /// Convenience: pin `src` to the right of `dst` with an optional gap,
    /// and top-align them.
    pub fn right_of(&mut self, src: AnchorId, dst: AnchorId, gap: i16) {
        self.constrain(src, Edge::Left, dst, Edge::Right, gap);
        self.constrain(src, Edge::Top,  dst, Edge::Top,   0);
    }

    /// Convenience: align the left edges of `src` and `dst`.
    pub fn align_left(&mut self, src: AnchorId, dst: AnchorId) {
        self.constrain(src, Edge::Left, dst, Edge::Left, 0);
    }

    /// Convenience: align the right edges of `src` and `dst`.
    pub fn align_right(&mut self, src: AnchorId, dst: AnchorId) {
        self.constrain(src, Edge::Right, dst, Edge::Right, 0);
    }

    // ------------------------------------------------------------------
    // Resolution
    // ------------------------------------------------------------------

    /// Compute concrete `Rect` positions for all widgets.
    ///
    /// Uses iterative forward resolution: each pass applies every constraint
    /// that has a fully-known source value.  Stops when no progress is made
    /// or `MAX_PASSES` is exhausted.
    pub fn resolve(&mut self) {
        self.resolved.clear();

        // Seed the container rect.
        let container = Resolved {
            top:    Some(0),
            left:   Some(0),
            bottom: Some(self.height as i16),
            right:  Some(self.width as i16),
        };
        self.resolved.insert(Self::CONTAINER, container);

        // Initialise all slots with empty Resolved entries.
        for id in self.slots.keys() {
            self.resolved.entry(*id).or_default();
        }

        for _pass in 0..Self::MAX_PASSES {
            let mut progress = false;

            for &Constraint { src, src_edge, dst, dst_edge, offset } in &self.constraints {
                // Get the source value from `dst`'s resolved state.
                let src_value = {
                    let dst_res = match self.resolved.get(&dst) {
                        Some(r) => *r,
                        None    => continue,
                    };
                    match dst_edge {
                        Edge::Top    => dst_res.top.map(|v| v + offset),
                        Edge::Left   => dst_res.left.map(|v| v + offset),
                        Edge::Bottom => dst_res.bottom.map(|v| v + offset),
                        Edge::Right  => dst_res.right.map(|v| v + offset),
                    }
                };

                let src_value = match src_value {
                    Some(v) => v,
                    None    => continue,
                };

                let entry = self.resolved.entry(src).or_default();
                let field = match src_edge {
                    Edge::Top    => &mut entry.top,
                    Edge::Left   => &mut entry.left,
                    Edge::Bottom => &mut entry.bottom,
                    Edge::Right  => &mut entry.right,
                };

                if field.is_none() {
                    *field = Some(src_value);
                    progress = true;
                }
            }

            // After applying explicit constraints, derive opposite edges
            // from the preferred size where one edge is known but the other isn't.
            // E.g. if `top` is known but `bottom` is not, set bottom = top + pref_h.
            // This allows `below()` chains to work without explicit bottom constraints.
            let ids: Vec<AnchorId> = self.resolved.keys()
                .copied()
                .filter(|&id| id != Self::CONTAINER)
                .collect();
            for id in ids {
                let (pw, ph) = self.slots.get(&id)
                    .map(|s| (s.pref_w as i16, s.pref_h as i16))
                    .unwrap_or((20, 10));
                let entry = self.resolved.entry(id).or_default();
                if entry.bottom.is_none() {
                    if let Some(t) = entry.top { entry.bottom = Some(t + ph); progress = true; }
                }
                if entry.top.is_none() {
                    if let Some(b) = entry.bottom { entry.top = Some(b - ph); progress = true; }
                }
                if entry.right.is_none() {
                    if let Some(l) = entry.left { entry.right = Some(l + pw); progress = true; }
                }
                if entry.left.is_none() {
                    if let Some(r) = entry.right { entry.left = Some(r - pw); progress = true; }
                }
            }

            if !progress { break; }
        }
    }

    /// Return the resolved `Rect` for a widget, or a fallback at `(0,0)`.
    pub fn rect_of(&mut self, id: AnchorId) -> crate::layout::Rect {
        if self.resolved.is_empty() { self.resolve(); }
        let slot = self.slots.get(&id);
        let (pw, ph) = slot.map(|s| (s.pref_w, s.pref_h)).unwrap_or((20, 10));
        self.resolved.get(&id).copied().unwrap_or_default().to_rect(pw, ph)
    }

    // ------------------------------------------------------------------
    // Rendering
    // ------------------------------------------------------------------

    /// Resolve all positions and render every widget into `canvas`.
    ///
    /// Widgets are rendered in insertion order (the order `add()` was called).
    /// Later widgets paint over earlier ones if they overlap.
    pub fn render(&mut self, canvas: &mut SubCanvas<'_>) {
        self.resolve();

        // Collect sorted ids (insertion order via next_id sequence).
        let mut ids: Vec<AnchorId> = self.slots.keys().copied().collect();
        ids.sort_by_key(|id| id.0);

        // Temporarily take slots out to avoid borrow conflicts.
        let mut render_list: Vec<(crate::layout::Rect, AnchorId)> = Vec::new();
        for id in &ids {
            let slot = self.slots.get(id).unwrap();
            let rect = self.resolved.get(id).copied().unwrap_or_default()
                .to_rect(slot.pref_w, slot.pref_h);
            render_list.push((rect, *id));
        }

        for (rect, id) in render_list {
            if rect.width == 0 || rect.height == 0 { continue; }
            let x = rect.x.min(canvas.width().saturating_sub(1));
            let y = rect.y.min(canvas.height().saturating_sub(1));
            let w = rect.width.min(canvas.width().saturating_sub(x));
            let h = rect.height.min(canvas.height().saturating_sub(y));
            if w == 0 || h == 0 { continue; }

            if let Some(slot) = self.slots.get(&id) {
                // Safety: we need immutable access to slot.widget and mutable
                // access to canvas.  Use a raw pointer to work around the
                // borrow checker — we never alias the widget itself.
                let widget_ptr: *const dyn Widget = slot.widget.as_ref();
                let mut sub = canvas.sub(x, y, w, h);
                // SAFETY: widget is not mutated during render.
                unsafe { (*widget_ptr).render(&mut sub); }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::Panel;

    #[test]
    fn basic_constraint_resolution() {
        let mut layout = AnchorLayout::new(80, 24);
        let a = layout.add(Box::new(Panel::new().title("A")), Some((20, 5)));

        layout.constrain(a, Edge::Top,  AnchorLayout::CONTAINER, Edge::Top,  2);
        layout.constrain(a, Edge::Left, AnchorLayout::CONTAINER, Edge::Left, 4);
        layout.resolve();

        let rect = layout.rect_of(a);
        assert_eq!(rect.y, 2);
        assert_eq!(rect.x, 4);
        assert_eq!(rect.width, 20);
        assert_eq!(rect.height, 5);
    }

    #[test]
    fn daisy_chain_below() {
        let mut layout = AnchorLayout::new(80, 40);
        let a = layout.add(Box::new(Panel::new().title("A")), Some((30, 5)));
        let b = layout.add(Box::new(Panel::new().title("B")), Some((30, 5)));
        let c = layout.add(Box::new(Panel::new().title("C")), Some((30, 5)));

        layout.constrain(a, Edge::Top,  AnchorLayout::CONTAINER, Edge::Top,  1);
        layout.constrain(a, Edge::Left, AnchorLayout::CONTAINER, Edge::Left, 1);
        layout.below(b, a, 1);
        layout.below(c, b, 1);

        layout.resolve();

        let ra = layout.rect_of(a);
        let rb = layout.rect_of(b);
        let rc = layout.rect_of(c);

        assert_eq!(ra.y, 1);
        assert_eq!(rb.y, ra.y + ra.height + 1);
        assert_eq!(rc.y, rb.y + rb.height + 1);
        assert_eq!(ra.x, rb.x);  // left-aligned
        assert_eq!(rb.x, rc.x);
    }

    #[test]
    fn fill_constraint() {
        let mut layout = AnchorLayout::new(80, 24);
        let a = layout.add(Box::new(Panel::new().title("Fill")), None);
        layout.fill(a, 2);
        layout.resolve();

        let r = layout.rect_of(a);
        assert_eq!(r.x, 2);
        assert_eq!(r.y, 2);
        assert_eq!(r.width,  80 - 4);
        assert_eq!(r.height, 24 - 4);
    }

    #[test]
    fn centre_h() {
        let mut layout = AnchorLayout::new(80, 24);
        let a = layout.add(Box::new(Panel::new().title("Centred")), Some((20, 5)));
        layout.constrain_centre_h(a);
        layout.resolve();

        let r = layout.rect_of(a);
        assert_eq!(r.x, 30);   // (80 - 20) / 2
        assert_eq!(r.width, 20);
    }
}
