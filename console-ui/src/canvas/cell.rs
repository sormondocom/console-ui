use crate::color::{Color, StyleFlags};

/// A single terminal cell: one character + foreground color + background color
/// + style attributes.
///
/// Wide characters (East Asian CJK, etc.) occupy two columns.  When a wide
/// character is placed at column `c`, the cell at column `c+1` is set to
/// `wide_continuation: true` with a space character.  The renderer skips
/// emitting a cursor-move + character for continuation cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch:    char,
    pub fg:    Color,
    pub bg:    Color,
    pub style: StyleFlags,
    pub wide_continuation: bool,
}

impl Cell {
    /// A blank cell with all attributes reset to defaults.
    pub const BLANK: Cell = Cell {
        ch:    ' ',
        fg:    Color::Default,
        bg:    Color::Default,
        style: StyleFlags::empty(),
        wide_continuation: false,
    };
}

impl Default for Cell {
    fn default() -> Self {
        Self::BLANK
    }
}
