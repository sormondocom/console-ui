use crate::canvas::SubCanvas;
use crate::color::{BasicColor, Color, StyleFlags};
use crate::event::Key;
use crate::widget::traits::{InteractiveWidget, Widget};

const RESIZE_STEP: f32 = 0.05;

/// Which pane of a four-quadrant layout is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pane4 {
    #[default] TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// 2×2 grid layout with four independently resizable panes.
///
/// `h_ratio` controls where the vertical divider sits (0.0–1.0 fraction of width).
/// `v_ratio` controls where the horizontal divider sits (fraction of height).
///
/// Press `Tab` / `Shift-Tab` to cycle focus between panes.
/// Press `r` to enter resize mode; then use arrow keys to move dividers.
pub struct Split4 {
    pub h_ratio:      f32,
    pub v_ratio:      f32,
    pub focused_pane: Pane4,
    pub resize_mode:  bool,
    pub panes:        [Box<dyn Widget>; 4],  // [TL, TR, BL, BR]
}

impl Split4 {
    pub fn new(
        top_left:     Box<dyn Widget>,
        top_right:    Box<dyn Widget>,
        bottom_left:  Box<dyn Widget>,
        bottom_right: Box<dyn Widget>,
    ) -> Self {
        Self {
            h_ratio: 0.5, v_ratio: 0.5,
            focused_pane: Pane4::TopLeft, resize_mode: false,
            panes: [top_left, top_right, bottom_left, bottom_right],
        }
    }

    fn pane_idx(pane: Pane4) -> usize {
        match pane { Pane4::TopLeft => 0, Pane4::TopRight => 1, Pane4::BottomLeft => 2, Pane4::BottomRight => 3 }
    }
}

impl Widget for Split4 {
    fn render(&self, canvas: &mut SubCanvas<'_>) {
        let w = canvas.width();
        let h = canvas.height();
        if w < 5 || h < 5 { return; }

        let div_col = ((w as f32 * self.h_ratio) as u16).clamp(2, w.saturating_sub(2));
        let div_row = ((h as f32 * self.v_ratio) as u16).clamp(2, h.saturating_sub(2));

        let div_color = if self.resize_mode {
            Color::Basic(BasicColor::Yellow, true)
        } else {
            Color::Default
        };

        // ── Quadrant sub-canvases ─────────────────────────────────────────────
        let right_w  = w.saturating_sub(div_col + 1);
        let bottom_h = h.saturating_sub(div_row + 1);

        {
            let mut sub = canvas.sub(0, 0, div_col, div_row);
            self.panes[0].render(&mut sub);
        }
        {
            let mut sub = canvas.sub(div_col + 1, 0, right_w, div_row);
            self.panes[1].render(&mut sub);
        }
        {
            let mut sub = canvas.sub(0, div_row + 1, div_col, bottom_h);
            self.panes[2].render(&mut sub);
        }
        {
            let mut sub = canvas.sub(div_col + 1, div_row + 1, right_w, bottom_h);
            self.panes[3].render(&mut sub);
        }

        // ── Divider lines ─────────────────────────────────────────────────────
        // Vertical divider
        canvas.fill_v(div_col, 0, h, '│', div_color, Color::Default);
        // Horizontal divider
        canvas.fill_h(0, div_row, w, '─', div_color, Color::Default);
        // Cross-point
        canvas.set(div_col, div_row, crate::canvas::Cell {
            ch: '┼', fg: div_color, bg: Color::Default,
            style: StyleFlags::empty(), wide_continuation: false,
        });

        // ── Focus indicator ───────────────────────────────────────────────────
        // Draw a subtle corner highlight on the focused pane's top-left corner.
        let (focus_col, focus_row) = match self.focused_pane {
            Pane4::TopLeft     => (0, 0),
            Pane4::TopRight    => (div_col + 1, 0),
            Pane4::BottomLeft  => (0, div_row + 1),
            Pane4::BottomRight => (div_col + 1, div_row + 1),
        };
        if self.focused_pane != Pane4::TopLeft || self.resize_mode {
            canvas.set(focus_col, focus_row, crate::canvas::Cell {
                ch: '●',
                fg: Color::Basic(BasicColor::Cyan, true),
                bg: Color::Default,
                style: StyleFlags::empty(),
                wide_continuation: false,
            });
        }
    }

    fn min_size(&self) -> (u16, u16) { (10, 8) }
}

impl InteractiveWidget for Split4 {
    fn handle_key(&mut self, key: Key) -> bool {
        if self.resize_mode {
            match key {
                Key::Left  => { self.h_ratio = (self.h_ratio - RESIZE_STEP).max(0.1); return true; }
                Key::Right => { self.h_ratio = (self.h_ratio + RESIZE_STEP).min(0.9); return true; }
                Key::Up    => { self.v_ratio = (self.v_ratio - RESIZE_STEP).max(0.1); return true; }
                Key::Down  => { self.v_ratio = (self.v_ratio + RESIZE_STEP).min(0.9); return true; }
                Key::Escape | Key::Char('r') => { self.resize_mode = false; return true; }
                _ => {}
            }
        }
        match key {
            Key::Tab => {
                self.focused_pane = match self.focused_pane {
                    Pane4::TopLeft     => Pane4::TopRight,
                    Pane4::TopRight    => Pane4::BottomRight,
                    Pane4::BottomRight => Pane4::BottomLeft,
                    Pane4::BottomLeft  => Pane4::TopLeft,
                };
                true
            }
            Key::BackTab => {
                self.focused_pane = match self.focused_pane {
                    Pane4::TopLeft     => Pane4::BottomLeft,
                    Pane4::BottomLeft  => Pane4::BottomRight,
                    Pane4::BottomRight => Pane4::TopRight,
                    Pane4::TopRight    => Pane4::TopLeft,
                };
                true
            }
            Key::Char('r') => { self.resize_mode = !self.resize_mode; true }
            _ => false,
        }
    }

    fn is_focused(&self) -> bool { true }
    fn set_focused(&mut self, _: bool) {}
}
