use std::sync::OnceLock;

static CAPS: OnceLock<TermCaps> = OnceLock::new();

/// Detected color depth of the current terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ColorLevel {
    /// No color support — plain characters only (dumb terminal, pipe, DOS fallback).
    #[default]
    None,
    /// VT-100 / ANSI basic 8-color + bright variants via SGR 30–37 / 90–97.
    Vt100Basic,
    /// xterm 256-color palette via SGR 38;5;N.
    Ansi256,
    /// 24-bit true color via SGR 38;2;R;G;B.
    TrueColor,
}

/// Snapshot of terminal capabilities detected at startup.
#[derive(Debug, Clone)]
pub struct TermCaps {
    pub color_level: ColorLevel,
    /// Terminal can render Unicode box-drawing and other non-ASCII glyphs.
    pub unicode: bool,
    pub cols: u16,
    pub rows: u16,
}

impl TermCaps {
    /// Probe the running environment and return detected capabilities.
    ///
    /// Detection is intentionally conservative: we only raise the level when
    /// there is a strong positive signal, so an unknown terminal gets a safe
    /// baseline rather than broken escape sequences.
    pub fn detect() -> Self {
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));

        // $NO_COLOR or dumb terminal → strip everything.
        if std::env::var_os("NO_COLOR").is_some() {
            return Self { color_level: ColorLevel::None, unicode: false, cols, rows };
        }

        let term = std::env::var("TERM").unwrap_or_default();
        if term == "dumb" || term.is_empty() {
            // On non-TTY / piped output we still want ASCII-safe rendering.
            let unicode = Self::detect_unicode();
            return Self { color_level: ColorLevel::None, unicode, cols, rows };
        }

        let color_level = Self::detect_color(&term);
        let unicode     = Self::detect_unicode();

        Self { color_level, unicode, cols, rows }
    }

    fn detect_color(term: &str) -> ColorLevel {
        // Strongest signal: explicit $COLORTERM declaration.
        let colorterm = std::env::var("COLORTERM").unwrap_or_default();
        let colorterm  = colorterm.to_lowercase();
        if colorterm == "truecolor" || colorterm == "24bit" {
            return ColorLevel::TrueColor;
        }

        // Windows Terminal sets $WT_SESSION.
        if std::env::var_os("WT_SESSION").is_some() {
            return ColorLevel::TrueColor;
        }

        // Common 256-color $TERM values.
        if term.contains("256color") || term.contains("256colour") {
            return ColorLevel::Ansi256;
        }

        // Well-known true-color terminal programs.
        let term_program = std::env::var("TERM_PROGRAM").unwrap_or_default();
        match term_program.as_str() {
            "iTerm.app" | "Hyper" | "vscode" => return ColorLevel::TrueColor,
            "Apple_Terminal" => return ColorLevel::Ansi256,
            _ => {}
        }

        // xterm and rxvt families support at least 256 colors in modern builds.
        if term.starts_with("xterm") || term.starts_with("rxvt-unicode") {
            return ColorLevel::Ansi256;
        }

        // Safe fallback for any other recognisable ANSI terminal.
        ColorLevel::Vt100Basic
    }

    fn detect_unicode() -> bool {
        // Check locale environment variables for UTF-8 indication.
        for var in &["LC_ALL", "LC_CTYPE", "LANG"] {
            if let Ok(val) = std::env::var(var) {
                if val.to_uppercase().contains("UTF-8") || val.to_uppercase().contains("UTF8") {
                    return true;
                }
            }
        }
        // Windows: check for UTF-8 code page (65001) via crossterm / assumption.
        // On Windows 10+ with modern terminal, assume Unicode is available.
        #[cfg(target_os = "windows")]
        {
            // Windows Terminal or any WT_SESSION terminal supports Unicode.
            if std::env::var_os("WT_SESSION").is_some() {
                return true;
            }
        }
        false
    }

    /// Update terminal size (call after a SIGWINCH / Resize event).
    pub fn with_size(mut self, cols: u16, rows: u16) -> Self {
        self.cols = cols;
        self.rows = rows;
        self
    }
}

/// Initialise the global capability singleton.  Call once at program start.
/// Returns a reference to the stored value.
pub fn init_caps() -> &'static TermCaps {
    CAPS.get_or_init(TermCaps::detect)
}

/// Access the global capability singleton.
/// Panics if `init_caps()` has not been called yet.
pub fn caps() -> &'static TermCaps {
    CAPS.get().expect("TermCaps not initialised — call init_caps() first")
}
