use std::io::{self, Write};

use crossterm::{
    cursor,
    queue,
    style::{self, Attribute, Attributes, ContentStyle, StyledContent},
    terminal,
};

use crate::term::TermCaps;
use super::{Canvas, Cell};
use crate::color::StyleFlags;

/// Stateful double-buffer renderer.
///
/// Maintains a "previous frame" canvas.  On each call to `render()`, only the
/// cells that differ from the previous frame are re-emitted, minimising the
/// number of bytes written to stdout.
pub struct Renderer {
    prev:   Canvas,
    caps:   &'static TermCaps,
    stdout: io::Stdout,
}

impl Renderer {
    pub fn new(caps: &'static TermCaps) -> Self {
        Self {
            prev:   Canvas::new(caps.cols, caps.rows),
            caps,
            stdout: io::stdout(),
        }
    }

    /// Diff `next` against the previous frame and flush only changed cells.
    pub fn render(&mut self, next: &Canvas) -> std::io::Result<()> {
        // Hide cursor during redraw to prevent flickering.
        queue!(self.stdout, cursor::Hide)?;

        let width  = next.width();
        let height = next.height();

        for row in 0..height {
            for col in 0..width {
                let cell = next.get(col, row);
                let prev = self.prev.get(col, row);

                if cell == prev {
                    continue;
                }

                // Skip wide continuation cells — they were already covered by
                // the leading wide character's cell.
                if cell.wide_continuation {
                    continue;
                }

                queue!(self.stdout, cursor::MoveTo(col, row))?;
                self.emit_cell(&cell)?;
            }
        }

        // Reset attributes and show cursor.
        queue!(self.stdout, style::ResetColor, cursor::Show)?;
        self.stdout.flush()?;

        // Copy next → prev for the next diff pass.
        if self.prev.width() != width || self.prev.height() != height {
            self.prev = Canvas::new(width, height);
        }
        for row in 0..height {
            for col in 0..width {
                self.prev.set(col, row, next.get(col, row));
            }
        }

        Ok(())
    }

    /// Full redraw without diffing — used after a terminal resize.
    pub fn force_render(&mut self, next: &Canvas) -> std::io::Result<()> {
        // Invalidate prev so every cell is considered changed.
        self.prev = Canvas::new(0, 0);
        self.render(next)
    }

    /// Clear the physical screen and reset the prev buffer.
    pub fn clear_screen(&mut self) -> std::io::Result<()> {
        queue!(self.stdout, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;
        self.stdout.flush()?;
        self.prev = Canvas::new(0, 0);
        Ok(())
    }

    fn emit_cell(&mut self, cell: &Cell) -> std::io::Result<()> {
        let fg = cell.fg.downgrade(self.caps.color_level).to_crossterm();
        let bg = cell.bg.downgrade(self.caps.color_level).to_crossterm();

        let mut attrs = Attributes::default();
        if cell.style.contains(StyleFlags::BOLD)          { attrs.set(Attribute::Bold); }
        if cell.style.contains(StyleFlags::DIM)           { attrs.set(Attribute::Dim); }
        if cell.style.contains(StyleFlags::ITALIC)        { attrs.set(Attribute::Italic); }
        if cell.style.contains(StyleFlags::UNDERLINE)     { attrs.set(Attribute::Underlined); }
        if cell.style.contains(StyleFlags::BLINK)         { attrs.set(Attribute::SlowBlink); }
        if cell.style.contains(StyleFlags::REVERSE)       { attrs.set(Attribute::Reverse); }
        if cell.style.contains(StyleFlags::STRIKETHROUGH) { attrs.set(Attribute::CrossedOut); }

        let content_style = ContentStyle {
            foreground_color: Some(fg),
            background_color: Some(bg),
            underline_color:  None,
            attributes:       attrs,
        };

        let styled: StyledContent<char> = StyledContent::new(content_style, cell.ch);
        queue!(self.stdout, style::PrintStyledContent(styled))?;
        Ok(())
    }
}
