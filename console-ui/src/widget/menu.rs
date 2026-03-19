use crate::canvas::SubCanvas;
use crate::color::{Color, StyleFlags};
use crate::event::Key;
use super::traits::{InteractiveWidget, Widget};

/// A scrollable, keyboard-navigable list menu.
pub struct Menu {
    pub items:          Vec<String>,
    pub cursor:         usize,
    pub scroll_offset:  usize,
    pub focused:        bool,
    pub fg:             Color,
    pub bg:             Color,
    pub cursor_fg:      Color,
    pub cursor_bg:      Color,
    pub cursor_style:   StyleFlags,
    pub prefix_normal:  String,
    pub prefix_cursor:  String,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            items:         Vec::new(),
            cursor:        0,
            scroll_offset: 0,
            focused:       true,
            fg:            Color::Default,
            bg:            Color::Default,
            cursor_fg:     Color::Default,
            cursor_bg:     Color::Default,
            cursor_style:  StyleFlags::REVERSE,
            prefix_normal: "  ".into(),
            prefix_cursor: "> ".into(),
        }
    }
}

impl Menu {
    pub fn new(items: Vec<impl Into<String>>) -> Self {
        Self { items: items.into_iter().map(Into::into).collect(), ..Self::default() }
    }

    pub fn cursor_colors(mut self, fg: Color, bg: Color) -> Self {
        self.cursor_fg = fg; self.cursor_bg = bg; self
    }

    /// Currently selected item index.
    pub fn selected(&self) -> usize { self.cursor }

    /// Currently selected item text.
    pub fn selected_item(&self) -> Option<&str> {
        self.items.get(self.cursor).map(|s| s.as_str())
    }

    fn adjust_scroll(&mut self, visible: u16) {
        let v = visible as usize;
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + v {
            self.scroll_offset = self.cursor.saturating_sub(v - 1);
        }
    }
}

impl Widget for Menu {
    fn render(&self, canvas: &mut SubCanvas<'_>) {
        let w = canvas.width();
        let h = canvas.height();

        for screen_row in 0..h {
            let item_idx = self.scroll_offset + screen_row as usize;
            if item_idx >= self.items.len() { break; }

            let is_cursor = item_idx == self.cursor && self.focused;
            let prefix = if is_cursor { &self.prefix_cursor } else { &self.prefix_normal };
            let line = format!("{}{}", prefix, self.items[item_idx]);

            let (fg, bg, style) = if is_cursor {
                (self.cursor_fg, self.cursor_bg, self.cursor_style)
            } else {
                (self.fg, self.bg, StyleFlags::empty())
            };

            // Pad line to full width so background fills the row.
            let padded = format!("{:<width$}", line, width = w as usize);
            canvas.print(0, screen_row, &padded, fg, bg, style);
        }
    }

    fn min_size(&self) -> (u16, u16) {
        let max_w = self.items.iter().map(|s| s.len() + 2).max().unwrap_or(10) as u16;
        (max_w, 1)
    }

    fn preferred_size(&self) -> (u16, u16) {
        let max_w = self.items.iter().map(|s| s.len() + 2).max().unwrap_or(10) as u16;
        (max_w, self.items.len() as u16)
    }
}

impl InteractiveWidget for Menu {
    fn handle_key(&mut self, key: Key) -> bool {
        match key {
            Key::Up | Key::Char('k') => {
                if self.cursor > 0 { self.cursor -= 1; }
                true
            }
            Key::Down | Key::Char('j') => {
                if self.cursor + 1 < self.items.len() { self.cursor += 1; }
                true
            }
            Key::Home => { self.cursor = 0; true }
            Key::End  => { self.cursor = self.items.len().saturating_sub(1); true }
            Key::PageUp => {
                self.cursor = self.cursor.saturating_sub(10);
                true
            }
            Key::PageDown => {
                self.cursor = (self.cursor + 10).min(self.items.len().saturating_sub(1));
                true
            }
            _ => false,
        }
    }

    fn is_focused(&self) -> bool { self.focused }
    fn set_focused(&mut self, f: bool) { self.focused = f; }
}
