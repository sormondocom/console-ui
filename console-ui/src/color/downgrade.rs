use super::types::{BasicColor, Color};
use crate::term::ColorLevel;

// ---------------------------------------------------------------------------
// xterm 256-color palette (indices 0–255) as (R, G, B) tuples.
// Indices 0–15 are the system colors (terminal-defined); we use conventional
// xterm defaults.  Indices 16–231 are the 6×6×6 color cube.  232–255 are the
// grayscale ramp.
// ---------------------------------------------------------------------------

const PALETTE_256: [(u8, u8, u8); 256] = build_palette();

const fn build_palette() -> [(u8, u8, u8); 256] {
    let mut p = [(0u8, 0u8, 0u8); 256];
    // System colors (0–15) — standard xterm defaults.
    let system: [(u8, u8, u8); 16] = [
        (0,   0,   0),   // 0  Black
        (128, 0,   0),   // 1  Red
        (0,   128, 0),   // 2  Green
        (128, 128, 0),   // 3  Yellow
        (0,   0,   128), // 4  Blue
        (128, 0,   128), // 5  Magenta
        (0,   128, 128), // 6  Cyan
        (192, 192, 192), // 7  White
        (128, 128, 128), // 8  Bright Black
        (255, 0,   0),   // 9  Bright Red
        (0,   255, 0),   // 10 Bright Green
        (255, 255, 0),   // 11 Bright Yellow
        (0,   0,   255), // 12 Bright Blue
        (255, 0,   255), // 13 Bright Magenta
        (0,   255, 255), // 14 Bright Cyan
        (255, 255, 255), // 15 Bright White
    ];
    let mut i = 0usize;
    while i < 16 {
        p[i] = system[i];
        i += 1;
    }
    // 6×6×6 color cube (16–231).
    let levels: [u8; 6] = [0, 95, 135, 175, 215, 255];
    let mut r = 0usize;
    while r < 6 {
        let mut g = 0usize;
        while g < 6 {
            let mut b = 0usize;
            while b < 6 {
                p[16 + r * 36 + g * 6 + b] = (levels[r], levels[g], levels[b]);
                b += 1;
            }
            g += 1;
        }
        r += 1;
    }
    // Grayscale ramp (232–255): 8 → 238 in steps of 10.
    let mut k = 0usize;
    while k < 24 {
        let v = 8 + k as u8 * 10;
        p[232 + k] = (v, v, v);
        k += 1;
    }
    p
}

// ---------------------------------------------------------------------------
// Basic-color lookup table: for each 256-palette entry, which BasicColor is
// closest?  Pre-computed at compile time from the system colors (0–15) using
// the same squared-RGB distance used at runtime.
// ---------------------------------------------------------------------------

const BASIC_COLORS: [(u8, u8, u8); 8] = [
    (0,   0,   0),   // Black
    (128, 0,   0),   // Red
    (0,   128, 0),   // Green
    (128, 128, 0),   // Yellow
    (0,   0,   128), // Blue
    (128, 0,   128), // Magenta
    (0,   128, 128), // Cyan
    (192, 192, 192), // White
];

fn nearest_basic(r: u8, g: u8, b: u8) -> BasicColor {
    let mut best_idx = 0usize;
    let mut best_dist = u32::MAX;
    for (i, &(cr, cg, cb)) in BASIC_COLORS.iter().enumerate() {
        let dr = r as i32 - cr as i32;
        let dg = g as i32 - cg as i32;
        let db = b as i32 - cb as i32;
        let dist = (dr * dr + dg * dg + db * db) as u32;
        if dist < best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }
    match best_idx {
        0 => BasicColor::Black,
        1 => BasicColor::Red,
        2 => BasicColor::Green,
        3 => BasicColor::Yellow,
        4 => BasicColor::Blue,
        5 => BasicColor::Magenta,
        6 => BasicColor::Cyan,
        _ => BasicColor::White,
    }
}

fn nearest_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let mut best_idx = 0u8;
    let mut best_dist = u32::MAX;
    for (i, &(cr, cg, cb)) in PALETTE_256.iter().enumerate() {
        let dr = r as i32 - cr as i32;
        let dg = g as i32 - cg as i32;
        let db = b as i32 - cb as i32;
        let dist = (dr * dr + dg * dg + db * db) as u32;
        if dist < best_dist {
            best_dist = dist;
            best_idx = i as u8;
        }
    }
    best_idx
}

impl Color {
    /// Reduce this color so it is within the capabilities of `level`.
    /// Gracefully degrades: TrueColor → Ansi256 → Vt100Basic → Default.
    pub fn downgrade(self, level: ColorLevel) -> Color {
        match (self, level) {
            // Already at or below the target level — no change.
            (Color::Default, _)       => Color::Default,
            (Color::Basic(_, _), ColorLevel::None) => Color::Default,
            (Color::Basic(c, b), _)   => Color::Basic(c, b),

            // Ansi256 → must become Basic or dropped.
            (Color::Ansi256(i), ColorLevel::Vt100Basic) => {
                let (r, g, b) = PALETTE_256[i as usize];
                Color::Basic(nearest_basic(r, g, b), false)
            }
            (Color::Ansi256(_), ColorLevel::None) => Color::Default,
            (Color::Ansi256(i), _)    => Color::Ansi256(i),

            // TrueColor → Ansi256 or Basic or dropped.
            (Color::TrueColor(r, g, b), ColorLevel::TrueColor) => Color::TrueColor(r, g, b),
            (Color::TrueColor(r, g, b), ColorLevel::Ansi256)   => Color::Ansi256(nearest_ansi256(r, g, b)),
            (Color::TrueColor(r, g, b), ColorLevel::Vt100Basic) => Color::Basic(nearest_basic(r, g, b), false),
            (Color::TrueColor(_, _, _), ColorLevel::None)       => Color::Default,
        }
    }

    /// Convert to crossterm's `Color` type for use with the rendering layer.
    pub fn to_crossterm(self) -> crossterm::style::Color {
        use crossterm::style::Color as CT;
        match self {
            Color::Default               => CT::Reset,
            Color::Basic(c, false) => match c {
                BasicColor::Black   => CT::Black,
                BasicColor::Red     => CT::DarkRed,
                BasicColor::Green   => CT::DarkGreen,
                BasicColor::Yellow  => CT::DarkYellow,
                BasicColor::Blue    => CT::DarkBlue,
                BasicColor::Magenta => CT::DarkMagenta,
                BasicColor::Cyan    => CT::DarkCyan,
                BasicColor::White   => CT::Grey,
            },
            Color::Basic(c, true) => match c {
                BasicColor::Black   => CT::DarkGrey,
                BasicColor::Red     => CT::Red,
                BasicColor::Green   => CT::Green,
                BasicColor::Yellow  => CT::Yellow,
                BasicColor::Blue    => CT::Blue,
                BasicColor::Magenta => CT::Magenta,
                BasicColor::Cyan    => CT::Cyan,
                BasicColor::White   => CT::White,
            },
            Color::Ansi256(i)          => CT::AnsiValue(i),
            Color::TrueColor(r, g, b)  => CT::Rgb { r, g, b },
        }
    }
}
