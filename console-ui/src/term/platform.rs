/// Query the current terminal dimensions.  Returns `(cols, rows)`.
/// Falls back to `(80, 24)` if the query fails (e.g. piped output).
pub fn terminal_size() -> (u16, u16) {
    crossterm::terminal::size().unwrap_or((80, 24))
}
