use crossterm::{
    cursor,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, Write};

/// RAII guard that enters raw mode + alternate screen on construction and
/// restores the terminal on drop.  Wrap your main event loop inside this.
pub struct RawModeGuard {
    alternate: bool,
}

impl RawModeGuard {
    /// Enable raw mode and switch to the alternate screen buffer.
    pub fn enter() -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(cursor::Hide)?;
        stdout.flush()?;
        Ok(Self { alternate: true })
    }

    /// Enable raw mode without the alternate screen (inline rendering).
    pub fn enter_inline() -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self { alternate: false })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        if self.alternate {
            let mut stdout = io::stdout();
            let _ = stdout.execute(LeaveAlternateScreen);
            let _ = stdout.execute(cursor::Show);
            let _ = stdout.flush();
        }
    }
}
