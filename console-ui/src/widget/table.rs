use unicode_width::UnicodeWidthStr;

use crate::border::BorderStyle;
use crate::canvas::{Cell, SubCanvas};
use crate::color::{Color, StyleFlags};
use crate::term::caps;
use super::traits::Widget;
use super::panel::Align;

/// A bordered table with optional header row and auto-justified columns.
pub struct Table {
    pub headers:      Option<Vec<String>>,
    pub rows:         Vec<Vec<String>>,
    /// Per-column alignment.  The last entry is repeated for excess columns.
    pub col_align:    Vec<Align>,
    pub border_style: BorderStyle,
    pub border_fg:    Color,
    pub border_bg:    Color,
    pub header_fg:    Color,
    pub header_bg:    Color,
    pub header_style: StyleFlags,
    pub row_fg:       Color,
    pub row_bg:       Color,
    pub alt_row_bg:   Option<Color>,
    /// Current scroll offset (first visible data row).
    pub scroll_row:   usize,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            headers:      None,
            rows:         Vec::new(),
            col_align:    vec![Align::Left],
            border_style: BorderStyle::default(),
            border_fg:    Color::Default,
            border_bg:    Color::Default,
            header_fg:    Color::Default,
            header_bg:    Color::Default,
            header_style: StyleFlags::BOLD,
            row_fg:       Color::Default,
            row_bg:       Color::Default,
            alt_row_bg:   None,
            scroll_row:   0,
        }
    }
}

impl Table {
    pub fn new() -> Self { Self::default() }

    pub fn headers(mut self, h: Vec<impl Into<String>>) -> Self {
        self.headers = Some(h.into_iter().map(Into::into).collect()); self
    }
    pub fn rows(mut self, r: Vec<Vec<impl Into<String>>>) -> Self {
        self.rows = r.into_iter().map(|row| row.into_iter().map(Into::into).collect()).collect();
        self
    }
    pub fn border_style(mut self, s: BorderStyle) -> Self { self.border_style = s; self }
    pub fn border_fg(mut self, c: Color) -> Self { self.border_fg = c; self }
    pub fn col_align(mut self, a: Vec<Align>) -> Self { self.col_align = a; self }
    pub fn alt_row_bg(mut self, c: Color) -> Self { self.alt_row_bg = Some(c); self }

    /// Compute natural column widths (max content width per column).
    fn natural_widths(&self) -> Vec<usize> {
        let n_cols = self.col_count();
        let mut widths = vec![0usize; n_cols];

        if let Some(hdrs) = &self.headers {
            for (i, h) in hdrs.iter().enumerate() {
                if i < n_cols { widths[i] = widths[i].max(UnicodeWidthStr::width(h.as_str())); }
            }
        }
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < n_cols { widths[i] = widths[i].max(UnicodeWidthStr::width(cell.as_str())); }
            }
        }
        widths
    }

    fn col_count(&self) -> usize {
        let from_headers = self.headers.as_ref().map(|h| h.len()).unwrap_or(0);
        let from_rows    = self.rows.iter().map(|r| r.len()).max().unwrap_or(0);
        from_headers.max(from_rows).max(1)
    }

    fn col_align_for(&self, col: usize) -> Align {
        *self.col_align.get(col).or_else(|| self.col_align.last()).unwrap_or(&Align::Left)
    }

    /// Distribute available width among columns.  Each column gets at minimum
    /// 1 character.  If there's extra space, distribute proportionally to
    /// natural widths.  If there's not enough space, shrink widest columns first.
    fn compute_widths(&self, available: u16) -> Vec<u16> {
        let natural = self.natural_widths();
        let n = natural.len();
        if n == 0 { return vec![]; }

        // Separators: n+1 borders (left edge + divider per col + right edge).
        let sep_total = (n as u16 + 1).min(available);
        let content_space = available.saturating_sub(sep_total) as usize;

        let natural_sum: usize = natural.iter().sum();
        let mut widths = vec![1u16; n];

        if natural_sum <= content_space {
            // All columns fit at their natural width.
            for (i, &nw) in natural.iter().enumerate() {
                widths[i] = nw as u16;
            }
        } else if content_space >= n {
            // Scale down proportionally from widest to narrowest.
            let mut remaining = content_space;
            let mut sorted_idx: Vec<usize> = (0..n).collect();
            sorted_idx.sort_by(|&a, &b| natural[b].cmp(&natural[a]));

            for (k, &i) in sorted_idx.iter().enumerate() {
                let cols_left = n - k;
                let fair_share = remaining / cols_left;
                widths[i] = fair_share.min(natural[i]).max(1) as u16;
                remaining = remaining.saturating_sub(widths[i] as usize);
            }
        }
        // else: each column gets 1 (set above).

        widths
    }

    /// Justify a string into a field of width `w` using the given alignment.
    fn justify(s: &str, w: usize, align: Align) -> String {
        let vis_w = UnicodeWidthStr::width(s);
        if vis_w >= w {
            // Truncate to fit.
            let mut out = String::new();
            let mut used = 0;
            for ch in s.chars() {
                let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
                if used + cw > w { break; }
                out.push(ch);
                used += cw;
            }
            return out;
        }
        let pad = w - vis_w;
        match align {
            Align::Left   => format!("{}{}", s, " ".repeat(pad)),
            Align::Right  => format!("{}{}", " ".repeat(pad), s),
            Align::Center => {
                let lpad = pad / 2;
                let rpad = pad - lpad;
                format!("{}{}{}", " ".repeat(lpad), s, " ".repeat(rpad))
            }
        }
    }
}

impl Widget for Table {
    fn render(&self, canvas: &mut SubCanvas<'_>) {
        let w = canvas.width();
        let h = canvas.height();
        if w < 3 || h < 3 { return; }

        let style = self.border_style.with_caps(caps());
        let g     = style.glyphs();
        let fg    = self.border_fg;
        let bg    = self.border_bg;

        let col_widths = self.compute_widths(w);
        let n = col_widths.len();
        if n == 0 { return; }

        // Helper: build a horizontal rule across all columns.
        let h_rule = |left: char, mid: char, seg: char, right: char, widths: &[u16]| -> String {
            let mut s = String::new();
            s.push(left);
            for (i, &cw) in widths.iter().enumerate() {
                for _ in 0..cw + 2 { s.push(seg); }
                if i < widths.len() - 1 { s.push(mid); }
            }
            s.push(right);
            s
        };

        let border_cell = |ch| Cell { ch, fg, bg, style: StyleFlags::empty(), wide_continuation: false };

        // ── Top border ────────────────────────────────────────────────────────
        let top_rule = h_rule(g.top_left, g.tee_top, g.horizontal, g.top_right, &col_widths);
        canvas.print(0, 0, &top_rule, fg, bg, StyleFlags::empty());

        let mut cur_row = 1u16;

        // ── Header row ────────────────────────────────────────────────────────
        let has_header = self.headers.is_some();
        if let Some(hdrs) = &self.headers {
            if cur_row < h {
                // Left border
                canvas.set(0, cur_row, border_cell(g.vertical));
                let mut col_x = 1u16;
                for (i, &cw) in col_widths.iter().enumerate() {
                    let text = hdrs.get(i).map(|s| s.as_str()).unwrap_or("");
                    let justified = format!(" {} ", Self::justify(text, cw as usize, self.col_align_for(i)));
                    canvas.print(col_x, cur_row, &justified, self.header_fg, self.header_bg, self.header_style);
                    col_x += cw + 2;
                    if i < n - 1 {
                        canvas.set(col_x, cur_row, border_cell(g.vertical));
                        col_x += 1;
                    }
                }
                canvas.set(col_x, cur_row, border_cell(g.vertical));
                cur_row += 1;
            }
            // Separator after header
            if cur_row < h {
                let sep = h_rule(g.tee_left, g.cross, g.horizontal, g.tee_right, &col_widths);
                canvas.print(0, cur_row, &sep, fg, bg, StyleFlags::empty());
                cur_row += 1;
            }
        }

        // ── Data rows ─────────────────────────────────────────────────────────
        let bottom_reserve = 1u16; // space for bottom border
        let data_rows_visible = (h.saturating_sub(cur_row).saturating_sub(bottom_reserve)) as usize;
        let start = self.scroll_row.min(self.rows.len().saturating_sub(1));
        let end   = (start + data_rows_visible).min(self.rows.len());

        for (row_idx, row) in self.rows[start..end].iter().enumerate() {
            if cur_row >= h.saturating_sub(1) { break; }

            let row_bg = if row_idx % 2 == 1 { self.alt_row_bg.unwrap_or(self.row_bg) } else { self.row_bg };

            canvas.set(0, cur_row, border_cell(g.vertical));
            let mut col_x = 1u16;
            for (i, &cw) in col_widths.iter().enumerate() {
                let text = row.get(i).map(|s| s.as_str()).unwrap_or("");
                let justified = format!(" {} ", Self::justify(text, cw as usize, self.col_align_for(i)));
                canvas.print(col_x, cur_row, &justified, self.row_fg, row_bg, StyleFlags::empty());
                col_x += cw + 2;
                if i < n - 1 {
                    canvas.set(col_x, cur_row, border_cell(g.vertical));
                    col_x += 1;
                }
            }
            canvas.set(col_x, cur_row, border_cell(g.vertical));

            // Row separator (only between rows, not after last)
            cur_row += 1;
            let is_last = row_idx + 1 >= end - start;
            if !is_last && cur_row < h.saturating_sub(1) {
                let sep = h_rule(g.tee_left, g.cross, g.horizontal, g.tee_right, &col_widths);
                canvas.print(0, cur_row, &sep, fg, bg, StyleFlags::empty());
                cur_row += 1;
            }
        }

        // ── Bottom border ─────────────────────────────────────────────────────
        let bot_row = h - 1;
        if cur_row <= bot_row {
            // Fill any gap between last data row and bottom.
            for r in cur_row..bot_row {
                canvas.set(0, r, border_cell(g.vertical));
                canvas.set(w - 1, r, border_cell(g.vertical));
            }
            let bot_rule = h_rule(g.bottom_left, g.tee_bottom, g.horizontal, g.bottom_right, &col_widths);
            canvas.print(0, bot_row, &bot_rule, fg, bg, StyleFlags::empty());
        }

        let _ = has_header;
    }

    fn min_size(&self) -> (u16, u16) {
        let n = self.col_count() as u16;
        // Minimum: each column 3 wide (1 char + 2 padding), separators, 3 rows.
        (n * 5 + 1, 3 + if self.headers.is_some() { 2 } else { 0 })
    }

    fn preferred_size(&self) -> (u16, u16) {
        let nat = self.natural_widths();
        let total_w: u16 = nat.iter().map(|&w| w as u16 + 2).sum::<u16>() + nat.len() as u16 + 1;
        let rows = self.rows.len() as u16 + if self.headers.is_some() { 2 } else { 0 } + 2;
        (total_w, rows)
    }
}
