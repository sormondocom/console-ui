use crate::border::BorderStyle;
use crate::canvas::{Cell, SubCanvas};
use crate::color::{Color, StyleFlags};
use crate::term::caps;
use super::traits::Widget;

/// Horizontal alignment for the panel title.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// A bordered box with an optional title.
///
/// The panel draws its border, positions the title, then delegates rendering
/// of its interior to an optional child widget.
pub struct Panel {
    pub title:        Option<String>,
    pub title_align:  Align,
    pub border_style: BorderStyle,
    pub border_fg:    Color,
    pub border_bg:    Color,
    pub title_fg:     Color,
    pub title_bg:     Color,
    pub title_style:  StyleFlags,
    pub inner_bg:     Color,
    pub child:        Option<Box<dyn Widget>>,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            title:        None,
            title_align:  Align::Left,
            border_style: BorderStyle::default(),
            border_fg:    Color::Default,
            border_bg:    Color::Default,
            title_fg:     Color::Default,
            title_bg:     Color::Default,
            title_style:  StyleFlags::BOLD,
            inner_bg:     Color::Default,
            child:        None,
        }
    }
}

impl Panel {
    pub fn new() -> Self { Self::default() }

    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into()); self
    }
    pub fn title_align(mut self, a: Align) -> Self { self.title_align = a; self }
    pub fn border_style(mut self, s: BorderStyle) -> Self { self.border_style = s; self }
    pub fn border_fg(mut self, c: Color) -> Self { self.border_fg = c; self }
    pub fn border_bg(mut self, c: Color) -> Self { self.border_bg = c; self }
    pub fn title_fg(mut self, c: Color) -> Self { self.title_fg = c; self }
    pub fn inner_bg(mut self, c: Color) -> Self { self.inner_bg = c; self }
    pub fn child(mut self, w: Box<dyn Widget>) -> Self { self.child = Some(w); self }
}

impl Widget for Panel {
    fn render(&self, canvas: &mut SubCanvas<'_>) {
        let w = canvas.width();
        let h = canvas.height();
        if w < 2 || h < 2 { return; }

        let style = self.border_style.with_caps(caps());
        let g = style.glyphs();
        let fg = self.border_fg;
        let bg = self.border_bg;

        let border_cell = |ch| Cell { ch, fg, bg, style: StyleFlags::empty(), wide_continuation: false };

        // ── Top edge ──────────────────────────────────────────────────────────
        canvas.set(0, 0, border_cell(g.top_left));
        canvas.fill_h(1, 0, w - 2, g.horizontal, fg, bg);
        canvas.set(w - 1, 0, border_cell(g.top_right));

        // ── Title on top edge ─────────────────────────────────────────────────
        if let Some(title) = &self.title {
            let inner_w = (w as usize).saturating_sub(4); // space for separators + 1 pad each side
            let display: String = title.chars().take(inner_w).collect();
            let title_len = display.chars().count() as u16 + 2; // +2 for spaces

            let title_col = match self.title_align {
                Align::Left   => 1u16,
                Align::Center => ((w.saturating_sub(title_len)) / 2).max(1),
                Align::Right  => w.saturating_sub(title_len + 1).max(1),
            };

            // Left separator
            canvas.set(title_col, 0, border_cell(g.title_left));
            // Space + title + space
            let label = format!(" {} ", display);
            canvas.print(
                title_col + 1, 0,
                &label,
                self.title_fg, self.title_bg, self.title_style,
            );
            // Right separator
            let right_sep = (title_col + 1 + label.chars().count() as u16).min(w - 1);
            canvas.set(right_sep, 0, border_cell(g.title_right));
        }

        // ── Sides and inner fill ───────────────────────────────────────────────
        for r in 1..h - 1 {
            canvas.set(0, r, border_cell(g.vertical));
            if self.inner_bg != Color::Default {
                canvas.fill_h(1, r, w - 2, ' ', Color::Default, self.inner_bg);
            }
            canvas.set(w - 1, r, border_cell(g.vertical));
        }

        // ── Bottom edge ───────────────────────────────────────────────────────
        canvas.set(0, h - 1, border_cell(g.bottom_left));
        canvas.fill_h(1, h - 1, w - 2, g.horizontal, fg, bg);
        canvas.set(w - 1, h - 1, border_cell(g.bottom_right));

        // ── Child widget in inner area ────────────────────────────────────────
        if let Some(child) = &self.child {
            let mut inner = canvas.sub(1, 1, w - 2, h - 2);
            child.render(&mut inner);
        }
    }

    fn min_size(&self) -> (u16, u16) {
        let (cw, ch) = self.child.as_ref().map(|c| c.min_size()).unwrap_or((0, 0));
        (cw + 2, ch + 2)
    }

    fn preferred_size(&self) -> (u16, u16) {
        let (cw, ch) = self.child.as_ref().map(|c| c.preferred_size()).unwrap_or((20, 10));
        (cw + 2, ch + 2)
    }
}
