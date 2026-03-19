use console_ui::prelude::*;
use super::Action;

const THEMES: &[(&str, Color, Color)] = &[
    ("Default",  Color::Default,                            Color::Default),
    ("Cyan",     Color::Basic(BasicColor::Cyan,    true),   Color::Default),
    ("Green",    Color::Basic(BasicColor::Green,   true),   Color::Default),
    ("Yellow",   Color::Basic(BasicColor::Yellow,  true),   Color::Default),
    ("Magenta",  Color::Basic(BasicColor::Magenta, true),   Color::Default),
    ("Red",      Color::Basic(BasicColor::Red,     true),   Color::Default),
    ("White",    Color::Basic(BasicColor::White,   true),   Color::Default),
];

pub struct BorderDemo {
    style_idx: usize,
    theme_idx: usize,
}

impl BorderDemo {
    pub fn new() -> Self { Self { style_idx: 0, theme_idx: 0 } }

    pub fn render(&self, canvas: &mut Canvas) {
        let w = canvas.width();
        let h = canvas.height();
        let mut root = canvas.sub(0, 0, w, h);

        let styles = BorderStyle::all();
        let style = styles[self.style_idx % styles.len()];
        let (theme_name, border_fg, _border_bg) = THEMES[self.theme_idx % THEMES.len()];

        // ── Outer wrapper ─────────────────────────────────────────────────────
        let outer = Panel::new()
            .title(" Border Style Preview ")
            .title_align(Align::Center)
            .border_style(BorderStyle::Single)
            .border_fg(Color::Basic(BasicColor::White, false));
        outer.render(&mut root);

        let inner_w = w.saturating_sub(2);
        let inner_h = h.saturating_sub(2);
        let mut inner = root.sub(1, 1, inner_w, inner_h);

        // Status line at top
        let status = format!(
            " Style: {:8}  Theme: {:8}  | ← / → change style   ↑ / ↓ change theme   Esc: back ",
            style.name(), theme_name
        );
        inner.print(0, 0, &status, Color::Basic(BasicColor::White, false), Color::Default, StyleFlags::DIM);

        // ── Preview box (centered) ────────────────────────────────────────────
        let box_w = inner_w.min(60);
        let box_h = inner_h.saturating_sub(3).min(20);
        let box_x = (inner_w.saturating_sub(box_w)) / 2;
        let box_y = 2u16;

        let preview = Panel::new()
            .title(format!(" {} style — {} theme ", style.name(), theme_name))
            .title_align(Align::Center)
            .border_style(style)
            .border_fg(border_fg);

        let mut preview_area = inner.sub(box_x, box_y, box_w, box_h);
        preview.render(&mut preview_area);

        // Sample content inside the preview box
        let sample_text = TextBlock::new(
            "This is a sample text block rendered inside the panel.\n\n\
             You can compose any widget as the child of a Panel.\n\n\
             Arrow keys cycle through styles and color themes.\n\n\
             The border gracefully degrades to ASCII (+, -, |) on\n\
             terminals that don't support Unicode box-drawing.",
        );
        let mut text_area = inner.sub(box_x + 2, box_y + 1, box_w.saturating_sub(4), box_h.saturating_sub(2));
        sample_text.render(&mut text_area);

        // ── Style selector row ────────────────────────────────────────────────
        let sel_y = box_y + box_h + 1;
        if sel_y < inner_h {
            let mut x = 2u16;
            for (i, s) in BorderStyle::all().iter().enumerate() {
                let label = format!(" {} ", s.name());
                let (fg, style_flag) = if i == self.style_idx % styles.len() {
                    (Color::Basic(BasicColor::Black, false), StyleFlags::REVERSE)
                } else {
                    (Color::Default, StyleFlags::empty())
                };
                inner.print(x, sel_y, &label, fg, Color::Default, style_flag);
                x += label.len() as u16 + 1;
            }
        }
    }

    pub fn handle_key(&mut self, key: Key) -> Action {
        match key {
            Key::Escape | Key::Char('q') => Action::GoTo(super::ScreenId::MainMenu),
            Key::Left  => { self.style_idx = self.style_idx.wrapping_sub(1).min(BorderStyle::all().len() - 1); Action::Continue }
            Key::Right => { self.style_idx = (self.style_idx + 1) % BorderStyle::all().len(); Action::Continue }
            Key::Up    => { self.theme_idx = self.theme_idx.wrapping_sub(1).min(THEMES.len() - 1); Action::Continue }
            Key::Down  => { self.theme_idx = (self.theme_idx + 1) % THEMES.len(); Action::Continue }
            _ => Action::Continue,
        }
    }
}
