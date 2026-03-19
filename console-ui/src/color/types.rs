use bitflags::bitflags;

/// A terminal color value.  Use `Color::downgrade()` before rendering to
/// ensure the value is within what the terminal can display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    /// Terminal default — emits SGR 39 (fg) / 49 (bg).
    #[default]
    Default,
    /// ANSI 8-color set + bright variants.
    Basic(BasicColor, bool /* bright */),
    /// xterm 256-color palette index (0–255).
    Ansi256(u8),
    /// 24-bit true color RGB.
    TrueColor(u8, u8, u8),
}

/// The 8 ANSI basic colors, in SGR order (30–37 for fg, 40–47 for bg).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BasicColor {
    Black   = 0,
    Red     = 1,
    Green   = 2,
    Yellow  = 3,
    Blue    = 4,
    Magenta = 5,
    Cyan    = 6,
    White   = 7,
}

bitflags! {
    /// Text attribute flags.  Combine with `|` for multi-attribute styles.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct StyleFlags: u8 {
        const BOLD          = 0b0000_0001;
        const DIM           = 0b0000_0010;
        const ITALIC        = 0b0000_0100;
        const UNDERLINE     = 0b0000_1000;
        const BLINK         = 0b0001_0000;
        const REVERSE       = 0b0010_0000;
        const STRIKETHROUGH = 0b0100_0000;
    }
}
