use crate::term::TermCaps;

/// Available border drawing styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderStyle {
    /// ASCII-safe: `+` corners, `-` horizontal, `|` vertical.
    /// Safe on every terminal including DOS / dumb / piped output.
    Ascii,
    /// Unicode single-line box drawing (U+2500 range).
    #[default]
    Single,
    /// Unicode double-line box drawing (U+2550 range).
    Double,
    /// Single-line with rounded corners (╭╮╰╯).
    Rounded,
    /// Heavy/thick single-line (U+2501 range).
    Heavy,
}

/// The eight glyphs that fully describe a rectangular border plus inline title
/// separators.
#[derive(Debug, Clone, Copy)]
pub struct BorderGlyphs {
    pub top_left:     char,
    pub top_right:    char,
    pub bottom_left:  char,
    pub bottom_right: char,
    pub horizontal:   char,
    pub vertical:     char,
    /// Left-side title separator (e.g. `├` or `+`).
    pub title_left:   char,
    /// Right-side title separator (e.g. `┤` or `+`).
    pub title_right:  char,
    /// Horizontal divider for use inside the box (e.g. row separators in Table).
    pub cross:        char,
    /// Tee pointing down (top of a divider).
    pub tee_top:      char,
    /// Tee pointing up (bottom of a divider).
    pub tee_bottom:   char,
    /// Tee pointing right (left edge of a divider).
    pub tee_left:     char,
    /// Tee pointing left (right edge of a divider).
    pub tee_right:    char,
}

impl BorderStyle {
    pub fn glyphs(self) -> BorderGlyphs {
        match self {
            BorderStyle::Ascii => BorderGlyphs {
                top_left: '+', top_right: '+', bottom_left: '+', bottom_right: '+',
                horizontal: '-', vertical: '|',
                title_left: '+', title_right: '+',
                cross: '+', tee_top: '+', tee_bottom: '+', tee_left: '+', tee_right: '+',
            },
            BorderStyle::Single => BorderGlyphs {
                top_left: '┌', top_right: '┐', bottom_left: '└', bottom_right: '┘',
                horizontal: '─', vertical: '│',
                title_left: '├', title_right: '┤',
                cross: '┼', tee_top: '┬', tee_bottom: '┴', tee_left: '├', tee_right: '┤',
            },
            BorderStyle::Double => BorderGlyphs {
                top_left: '╔', top_right: '╗', bottom_left: '╚', bottom_right: '╝',
                horizontal: '═', vertical: '║',
                title_left: '╠', title_right: '╣',
                cross: '╬', tee_top: '╦', tee_bottom: '╩', tee_left: '╠', tee_right: '╣',
            },
            BorderStyle::Rounded => BorderGlyphs {
                top_left: '╭', top_right: '╮', bottom_left: '╰', bottom_right: '╯',
                horizontal: '─', vertical: '│',
                title_left: '├', title_right: '┤',
                cross: '┼', tee_top: '┬', tee_bottom: '┴', tee_left: '├', tee_right: '┤',
            },
            BorderStyle::Heavy => BorderGlyphs {
                top_left: '┏', top_right: '┓', bottom_left: '┗', bottom_right: '┛',
                horizontal: '━', vertical: '┃',
                title_left: '┣', title_right: '┫',
                cross: '╋', tee_top: '┳', tee_bottom: '┻', tee_left: '┣', tee_right: '┫',
            },
        }
    }

    /// Automatically degrade to `Ascii` when the terminal doesn't support Unicode.
    pub fn with_caps(self, caps: &TermCaps) -> Self {
        if !caps.unicode && self != BorderStyle::Ascii {
            BorderStyle::Ascii
        } else {
            self
        }
    }

    /// Cycle to the next style (useful for interactive demos).
    pub fn next(self) -> Self {
        match self {
            BorderStyle::Ascii   => BorderStyle::Single,
            BorderStyle::Single  => BorderStyle::Double,
            BorderStyle::Double  => BorderStyle::Rounded,
            BorderStyle::Rounded => BorderStyle::Heavy,
            BorderStyle::Heavy   => BorderStyle::Ascii,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BorderStyle::Ascii   => "ASCII",
            BorderStyle::Single  => "Single",
            BorderStyle::Double  => "Double",
            BorderStyle::Rounded => "Rounded",
            BorderStyle::Heavy   => "Heavy",
        }
    }

    pub fn all() -> &'static [BorderStyle] {
        &[
            BorderStyle::Ascii,
            BorderStyle::Single,
            BorderStyle::Double,
            BorderStyle::Rounded,
            BorderStyle::Heavy,
        ]
    }
}
