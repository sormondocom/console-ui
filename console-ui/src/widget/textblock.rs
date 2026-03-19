use unicode_width::UnicodeWidthChar;

use crate::canvas::SubCanvas;
use crate::color::{Color, StyleFlags};
use super::traits::Widget;

/// How text should wrap when it exceeds the canvas width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WrapMode {
    /// Break at word boundaries (spaces).
    #[default]
    Word,
    /// Break at any character boundary.
    Char,
    /// No wrapping — long lines are truncated.
    None,
}

/// A scrollable block of word-wrapped text.
pub struct TextBlock {
    pub text:      String,
    pub fg:        Color,
    pub bg:        Color,
    pub style:     StyleFlags,
    pub wrap:      WrapMode,
    pub scroll_row: usize,
}

impl Default for TextBlock {
    fn default() -> Self {
        Self {
            text:      String::new(),
            fg:        Color::Default,
            bg:        Color::Default,
            style:     StyleFlags::empty(),
            wrap:      WrapMode::default(),
            scroll_row: 0,
        }
    }
}

impl TextBlock {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), ..Self::default() }
    }

    pub fn fg(mut self, c: Color) -> Self { self.fg = c; self }
    pub fn bg(mut self, c: Color) -> Self { self.bg = c; self }
    pub fn style(mut self, s: StyleFlags) -> Self { self.style = s; self }
    pub fn wrap(mut self, m: WrapMode) -> Self { self.wrap = m; self }

    /// Wrap `self.text` to `width` columns.  Hard newlines are always honoured.
    pub fn wrap_lines(&self, width: u16) -> Vec<String> {
        let w = width as usize;
        if w == 0 { return vec![]; }

        let mut lines = Vec::new();
        for hard_line in self.text.split('\n') {
            match self.wrap {
                WrapMode::None => { lines.push(hard_line.to_string()); }
                WrapMode::Char => char_wrap(hard_line, w, &mut lines),
                WrapMode::Word => word_wrap(hard_line, w, &mut lines),
            }
        }
        lines
    }
}

fn char_wrap(text: &str, width: usize, out: &mut Vec<String>) {
    let mut line = String::new();
    let mut line_w = 0;
    for ch in text.chars() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(1);
        if line_w + cw > width {
            out.push(std::mem::take(&mut line));
            line_w = 0;
        }
        line.push(ch);
        line_w += cw;
    }
    out.push(line);
}

fn word_wrap(text: &str, width: usize, out: &mut Vec<String>) {
    let mut line = String::new();
    let mut line_w = 0usize;

    for word in text.split_whitespace() {
        let word_w: usize = word.chars()
            .map(|c| UnicodeWidthChar::width(c).unwrap_or(1))
            .sum();

        if line_w + (if line_w > 0 { 1 } else { 0 }) + word_w > width {
            if line_w > 0 {
                out.push(std::mem::take(&mut line));
                line_w = 0;
            }
            // Word longer than width — fall back to char wrap.
            if word_w > width {
                char_wrap(word, width, out);
                continue;
            }
        }
        if line_w > 0 {
            line.push(' ');
            line_w += 1;
        }
        line.push_str(word);
        line_w += word_w;
    }
    out.push(line);
}

impl Widget for TextBlock {
    fn render(&self, canvas: &mut SubCanvas<'_>) {
        let w = canvas.width();
        let h = canvas.height();
        let lines = self.wrap_lines(w);

        let start = self.scroll_row.min(lines.len().saturating_sub(1));
        let end   = (start + h as usize).min(lines.len());

        for (screen_row, line) in lines[start..end].iter().enumerate() {
            canvas.print(0, screen_row as u16, line, self.fg, self.bg, self.style);
        }
    }

    fn min_size(&self) -> (u16, u16) { (10, 1) }

    fn preferred_size(&self) -> (u16, u16) {
        let lines = self.wrap_lines(80);
        let max_w = lines.iter().map(|l| l.len()).max().unwrap_or(20);
        (max_w as u16, lines.len() as u16)
    }
}
