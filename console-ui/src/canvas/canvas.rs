use unicode_width::UnicodeWidthChar;

use crate::color::{Color, StyleFlags};
use super::cell::Cell;

/// An off-screen rectangular cell buffer.
///
/// Coordinates are zero-based: (0, 0) is the top-left corner.
/// The buffer is stored row-major: `index = row * width + col`.
pub struct Canvas {
    width:  u16,
    height: u16,
    cells:  Vec<Cell>,
}

impl Canvas {
    /// Create a new canvas filled with blank cells.
    pub fn new(width: u16, height: u16) -> Self {
        let n = width as usize * height as usize;
        Self { width, height, cells: vec![Cell::BLANK; n] }
    }

    /// Resize the canvas, discarding existing content.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width  = width;
        self.height = height;
        let n = width as usize * height as usize;
        self.cells.clear();
        self.cells.resize(n, Cell::BLANK);
    }

    /// Fill the entire canvas with blank cells.
    pub fn clear(&mut self) {
        self.cells.fill(Cell::BLANK);
    }

    pub fn width(&self)  -> u16 { self.width }
    pub fn height(&self) -> u16 { self.height }
    pub fn cells(&self) -> &[Cell] { &self.cells }

    #[inline]
    fn idx(&self, col: u16, row: u16) -> Option<usize> {
        if col < self.width && row < self.height {
            Some(row as usize * self.width as usize + col as usize)
        } else {
            None
        }
    }

    /// Write a single cell.  Out-of-bounds writes are silently ignored.
    pub fn set(&mut self, col: u16, row: u16, cell: Cell) {
        if let Some(i) = self.idx(col, row) {
            // Handle wide characters: mark the next column as a continuation.
            let w = UnicodeWidthChar::width(cell.ch).unwrap_or(1);
            self.cells[i] = cell;
            if w == 2 {
                if let Some(j) = self.idx(col + 1, row) {
                    self.cells[j] = Cell {
                        ch: ' ',
                        fg: cell.fg,
                        bg: cell.bg,
                        style: cell.style,
                        wide_continuation: true,
                    };
                }
            }
        }
    }

    pub fn get(&self, col: u16, row: u16) -> Cell {
        self.idx(col, row).map(|i| self.cells[i]).unwrap_or(Cell::BLANK)
    }

    /// Write a string starting at `(col, row)`.  Wide characters automatically
    /// advance the cursor by 2 columns.  Text is clipped at the canvas edge.
    pub fn print(
        &mut self,
        col: u16, row: u16,
        s: &str,
        fg: Color, bg: Color, style: StyleFlags,
    ) {
        let mut c = col;
        for ch in s.chars() {
            if c >= self.width { break; }
            let w = UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
            self.set(c, row, Cell { ch, fg, bg, style, wide_continuation: false });
            c += w;
        }
    }

    /// Fill a horizontal span with a single character (e.g. border lines).
    pub fn fill_h(&mut self, col: u16, row: u16, len: u16, ch: char, fg: Color, bg: Color) {
        for c in col..col.saturating_add(len) {
            if c >= self.width { break; }
            self.set(c, row, Cell { ch, fg, bg, style: StyleFlags::empty(), wide_continuation: false });
        }
    }

    /// Fill a vertical span with a single character.
    pub fn fill_v(&mut self, col: u16, row: u16, len: u16, ch: char, fg: Color, bg: Color) {
        for r in row..row.saturating_add(len) {
            if r >= self.height { break; }
            self.set(col, r, Cell { ch, fg, bg, style: StyleFlags::empty(), wide_continuation: false });
        }
    }

    /// Return a `SubCanvas` view into a sub-rectangle of this canvas.
    /// Writes to the `SubCanvas` are translated into the parent's coordinate space.
    pub fn sub(&mut self, col: u16, row: u16, width: u16, height: u16) -> SubCanvas<'_> {
        // Clamp the sub-region to fit within self.
        let w = width.min(self.width.saturating_sub(col));
        let h = height.min(self.height.saturating_sub(row));
        SubCanvas {
            parent:     self,
            origin_col: col,
            origin_row: row,
            width:      w,
            height:     h,
        }
    }
}

// ---------------------------------------------------------------------------
// SubCanvas — a borrowed rectangular view into a parent Canvas
// ---------------------------------------------------------------------------

/// A borrowed view into a rectangular sub-region of a `Canvas`.
/// All coordinates are local (0, 0 = top-left of the sub-region).
pub struct SubCanvas<'a> {
    parent:     &'a mut Canvas,
    origin_col: u16,
    origin_row: u16,
    width:      u16,
    height:     u16,
}

impl<'a> SubCanvas<'a> {
    pub fn width(&self)  -> u16 { self.width }
    pub fn height(&self) -> u16 { self.height }

    pub fn set(&mut self, col: u16, row: u16, cell: Cell) {
        if col < self.width && row < self.height {
            self.parent.set(self.origin_col + col, self.origin_row + row, cell);
        }
    }

    pub fn get(&self, col: u16, row: u16) -> Cell {
        if col < self.width && row < self.height {
            self.parent.get(self.origin_col + col, self.origin_row + row)
        } else {
            Cell::BLANK
        }
    }

    pub fn print(&mut self, col: u16, row: u16, s: &str, fg: Color, bg: Color, style: StyleFlags) {
        if row >= self.height { return; }
        let mut c = col;
        for ch in s.chars() {
            if c >= self.width { break; }
            let w = UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
            self.set(c, row, Cell { ch, fg, bg, style, wide_continuation: false });
            c += w;
        }
    }

    pub fn fill_h(&mut self, col: u16, row: u16, len: u16, ch: char, fg: Color, bg: Color) {
        for c in col..col.saturating_add(len) {
            if c >= self.width { break; }
            self.set(c, row, Cell { ch, fg, bg, style: StyleFlags::empty(), wide_continuation: false });
        }
    }

    pub fn fill_v(&mut self, col: u16, row: u16, len: u16, ch: char, fg: Color, bg: Color) {
        for r in row..row.saturating_add(len) {
            if r >= self.height { break; }
            self.set(col, r, Cell { ch, fg, bg, style: StyleFlags::empty(), wide_continuation: false });
        }
    }

    pub fn clear(&mut self) {
        for r in 0..self.height {
            for c in 0..self.width {
                self.set(c, r, Cell::BLANK);
            }
        }
    }

    /// Create a nested sub-canvas within this sub-canvas.
    pub fn sub(&mut self, col: u16, row: u16, width: u16, height: u16) -> SubCanvas<'_> {
        let w = width.min(self.width.saturating_sub(col));
        let h = height.min(self.height.saturating_sub(row));
        SubCanvas {
            parent:     self.parent,
            origin_col: self.origin_col + col,
            origin_row: self.origin_row + row,
            width:      w,
            height:     h,
        }
    }
}
