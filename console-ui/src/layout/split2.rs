use crate::canvas::SubCanvas;
use crate::color::Color;
use crate::event::Key;
use crate::term::caps;
use crate::widget::traits::{InteractiveWidget, Widget};

const RESIZE_STEP: f32 = 0.05;

/// Which pane of a two-pane split is focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pane2 { #[default] First, Second }

/// Horizontal split: left pane | right pane.
pub struct HSplit {
    /// Divider position as fraction of total width (0.0–1.0).
    pub ratio:        f32,
    pub min_first:    u16,
    pub min_second:   u16,
    pub focused_pane: Pane2,
    pub first:        Box<dyn Widget>,
    pub second:       Box<dyn Widget>,
    /// When true, arrow keys resize the split; when false, they navigate within panes.
    pub resize_mode:  bool,
}

impl HSplit {
    pub fn new(first: Box<dyn Widget>, second: Box<dyn Widget>) -> Self {
        Self {
            ratio: 0.5, min_first: 4, min_second: 4,
            focused_pane: Pane2::First, first, second, resize_mode: false,
        }
    }
}

impl Widget for HSplit {
    fn render(&self, canvas: &mut SubCanvas<'_>) {
        let w = canvas.width();
        let h = canvas.height();
        if w < 3 { return; }

        let caps = caps();
        let div_ch = crate::border::BorderStyle::Single.with_caps(caps).glyphs().vertical;
        let div_col = compute_div(w, self.ratio, self.min_first, self.min_second);

        // Render first pane.
        {
            let mut sub = canvas.sub(0, 0, div_col, h);
            self.first.render(&mut sub);
        }
        // Divider column.
        let div_fg = if self.resize_mode {
            crate::color::Color::Basic(crate::color::BasicColor::Yellow, true)
        } else {
            Color::Default
        };
        canvas.fill_v(div_col, 0, h, div_ch, div_fg, Color::Default);

        // Render second pane.
        {
            let second_x = div_col + 1;
            let second_w = w.saturating_sub(second_x);
            let mut sub = canvas.sub(second_x, 0, second_w, h);
            self.second.render(&mut sub);
        }
    }

    fn min_size(&self) -> (u16, u16) {
        let (fw, fh) = self.first.min_size();
        let (sw, sh) = self.second.min_size();
        (fw + sw + 1, fh.max(sh))
    }
}

impl InteractiveWidget for HSplit {
    fn handle_key(&mut self, key: Key) -> bool {
        if self.resize_mode {
            match key {
                Key::Left  => { self.ratio = (self.ratio - RESIZE_STEP).max(0.1); return true; }
                Key::Right => { self.ratio = (self.ratio + RESIZE_STEP).min(0.9); return true; }
                Key::Escape | Key::Char('r') => { self.resize_mode = false; return true; }
                _ => {}
            }
        }
        match key {
            Key::Tab => {
                self.focused_pane = match self.focused_pane {
                    Pane2::First  => Pane2::Second,
                    Pane2::Second => Pane2::First,
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

/// Vertical split: top pane / bottom pane.
pub struct VSplit {
    pub ratio:        f32,
    pub min_first:    u16,
    pub min_second:   u16,
    pub focused_pane: Pane2,
    pub first:        Box<dyn Widget>,
    pub second:       Box<dyn Widget>,
    pub resize_mode:  bool,
}

impl VSplit {
    pub fn new(first: Box<dyn Widget>, second: Box<dyn Widget>) -> Self {
        Self {
            ratio: 0.5, min_first: 3, min_second: 3,
            focused_pane: Pane2::First, first, second, resize_mode: false,
        }
    }
}

impl Widget for VSplit {
    fn render(&self, canvas: &mut SubCanvas<'_>) {
        let w = canvas.width();
        let h = canvas.height();
        if h < 3 { return; }

        let caps = caps();
        let div_ch = crate::border::BorderStyle::Single.with_caps(caps).glyphs().horizontal;
        let div_row = compute_div(h, self.ratio, self.min_first, self.min_second);

        {
            let mut sub = canvas.sub(0, 0, w, div_row);
            self.first.render(&mut sub);
        }

        let div_fg = if self.resize_mode {
            crate::color::Color::Basic(crate::color::BasicColor::Yellow, true)
        } else {
            Color::Default
        };
        canvas.fill_h(0, div_row, w, div_ch, div_fg, Color::Default);

        {
            let second_y = div_row + 1;
            let second_h = h.saturating_sub(second_y);
            let mut sub = canvas.sub(0, second_y, w, second_h);
            self.second.render(&mut sub);
        }
    }

    fn min_size(&self) -> (u16, u16) {
        let (fw, fh) = self.first.min_size();
        let (sw, sh) = self.second.min_size();
        (fw.max(sw), fh + sh + 1)
    }
}

impl InteractiveWidget for VSplit {
    fn handle_key(&mut self, key: Key) -> bool {
        if self.resize_mode {
            match key {
                Key::Up   => { self.ratio = (self.ratio - RESIZE_STEP).max(0.1); return true; }
                Key::Down => { self.ratio = (self.ratio + RESIZE_STEP).min(0.9); return true; }
                Key::Escape | Key::Char('r') => { self.resize_mode = false; return true; }
                _ => {}
            }
        }
        match key {
            Key::Tab => {
                self.focused_pane = match self.focused_pane {
                    Pane2::First  => Pane2::Second,
                    Pane2::Second => Pane2::First,
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

fn compute_div(total: u16, ratio: f32, min_first: u16, min_second: u16) -> u16 {
    let desired = (total as f32 * ratio) as u16;
    desired.clamp(min_first, total.saturating_sub(min_second + 1))
}
