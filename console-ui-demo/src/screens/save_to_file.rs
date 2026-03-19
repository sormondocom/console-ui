use std::io::Write;
use std::path::PathBuf;

use console_ui::prelude::*;
use super::Action;

pub struct SaveToFile {
    pub path:   String,
    pub status: Option<String>,
    pub editing: bool,
}

impl SaveToFile {
    pub fn new() -> Self {
        Self {
            path:    "console-ui-snapshot.txt".into(),
            status:  None,
            editing: false,
        }
    }

    pub fn render(&self, canvas: &mut Canvas) {
        let w = canvas.width();
        let h = canvas.height();
        let mut root = canvas.sub(0, 0, w, h);

        let outer = Panel::new()
            .title(" Save View to File ")
            .title_align(Align::Center)
            .border_style(BorderStyle::Double)
            .border_fg(Color::Basic(BasicColor::Yellow, true));
        outer.render(&mut root);

        let inner_w = w.saturating_sub(2);
        let inner_h = h.saturating_sub(2);
        let mut inner = root.sub(1, 1, inner_w, inner_h);

        inner.print(0, 1,
            " Saves the current canvas as plain text (no escape codes).",
            Color::Default, Color::Default, StyleFlags::DIM);

        inner.print(0, 3, " Output file:", Color::Default, Color::Default, StyleFlags::BOLD);

        let path_display = format!(" {} ", self.path);
        let path_style = if self.editing { StyleFlags::REVERSE } else { StyleFlags::UNDERLINE };
        inner.print(16, 3, &path_display, Color::Basic(BasicColor::Cyan, true), Color::Default, path_style);

        if self.editing {
            inner.print(0, 5, " Type filename, Enter to confirm, Esc to cancel.",
                Color::Basic(BasicColor::Yellow, false), Color::Default, StyleFlags::DIM);
        } else {
            inner.print(0, 5, " e: edit path   Enter / s: save   Esc: back",
                Color::Default, Color::Default, StyleFlags::DIM);
        }

        if let Some(status) = &self.status {
            let fg = if status.starts_with("Saved") {
                Color::Basic(BasicColor::Green, true)
            } else {
                Color::Basic(BasicColor::Red, true)
            };
            inner.print(0, 7, status.as_str(), fg, Color::Default, StyleFlags::BOLD);
        }

        let _ = inner_h;
    }

    pub fn handle_key(&mut self, key: Key, canvas: &Canvas) -> Action {
        if self.editing {
            match key {
                Key::Enter  => { self.editing = false; }
                Key::Escape => { self.editing = false; }
                Key::Backspace => { self.path.pop(); }
                Key::Char(c) => { self.path.push(c); }
                _ => {}
            }
            return Action::Continue;
        }

        match key {
            Key::Escape | Key::Char('q') => Action::GoTo(super::ScreenId::MainMenu),
            Key::Char('e') => { self.editing = true; self.status = None; Action::Continue }
            Key::Enter | Key::Char('s') => {
                self.status = Some(match save_canvas_to_file(canvas, &self.path) {
                    Ok(_)  => format!("Saved {} rows to '{}'", canvas.height(), self.path),
                    Err(e) => format!("Error: {}", e),
                });
                Action::Continue
            }
            _ => Action::Continue,
        }
    }
}

/// Dump the canvas to a file as plain UTF-8 text, one row per line, no escape codes.
pub fn save_canvas_to_file(canvas: &Canvas, path: &str) -> std::io::Result<()> {
    let dest = PathBuf::from(path);
    let mut file = std::fs::File::create(&dest)?;

    for row in 0..canvas.height() {
        let mut line = String::with_capacity(canvas.width() as usize);
        for col in 0..canvas.width() {
            let cell = canvas.get(col, row);
            if cell.wide_continuation { continue; }
            line.push(cell.ch);
        }
        // Trim trailing spaces for cleaner output.
        let trimmed = line.trim_end();
        writeln!(file, "{}", trimmed)?;
    }

    Ok(())
}
