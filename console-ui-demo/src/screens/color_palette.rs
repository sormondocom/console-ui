use console_ui::prelude::*;
use super::Action;

pub struct ColorPalette {
    page: u8,
}

impl ColorPalette {
    pub fn new() -> Self { Self { page: 0 } }

    pub fn render(&self, canvas: &mut Canvas) {
        let w = canvas.width();
        let h = canvas.height();
        let mut root = canvas.sub(0, 0, w, h);

        let title = match self.page {
            0 => " Color Palette — ANSI Basic (page 1/3) ",
            1 => " Color Palette — xterm 256-color (page 2/3) ",
            _ => " Color Palette — True Color gradient (page 3/3) ",
        };
        let outer = Panel::new()
            .title(title)
            .title_align(Align::Center)
            .border_style(BorderStyle::Single)
            .border_fg(Color::Basic(BasicColor::White, false));
        outer.render(&mut root);

        let inner_w = w.saturating_sub(2);
        let inner_h = h.saturating_sub(2);
        let mut inner = root.sub(1, 1, inner_w, inner_h);

        inner.print(0, 0, " ← / → change page   Esc: back",
            Color::Default, Color::Default, StyleFlags::DIM);

        match self.page {
            0 => self.render_basic(&mut inner, inner_w, inner_h),
            1 => self.render_256(&mut inner, inner_w, inner_h),
            _ => self.render_truecolor(&mut inner, inner_w, inner_h),
        }
    }

    fn render_basic(&self, canvas: &mut SubCanvas<'_>, _w: u16, _h: u16) {
        let labels = ["Black","Red","Green","Yellow","Blue","Magenta","Cyan","White"];
        let colors = [
            BasicColor::Black, BasicColor::Red, BasicColor::Green, BasicColor::Yellow,
            BasicColor::Blue,  BasicColor::Magenta, BasicColor::Cyan, BasicColor::White,
        ];

        canvas.print(0, 2, " Normal colors:", Color::Default, Color::Default, StyleFlags::BOLD);
        canvas.print(0, 3, " Bright colors:", Color::Default, Color::Default, StyleFlags::BOLD);

        for (i, (&color, label)) in colors.iter().zip(labels.iter()).enumerate() {
            let col = 17 + i as u16 * 10;
            let block = format!(" {:<8}", label);
            canvas.print(col, 2, &block, Color::Basic(color, false), Color::Default, StyleFlags::empty());
            canvas.print(col, 3, &block, Color::Basic(color, true),  Color::Default, StyleFlags::empty());
        }

        // Show with background too
        canvas.print(0, 5, " Background:", Color::Default, Color::Default, StyleFlags::BOLD);
        for (i, &color) in colors.iter().enumerate() {
            let col = 14 + i as u16 * 10;
            let block = format!("  {:<8}", labels[i]);
            canvas.print(col, 5, &block, Color::Basic(BasicColor::White, true), Color::Basic(color, false), StyleFlags::empty());
        }

        // Style flags demo
        canvas.print(0, 8, " Style flags:", Color::Default, Color::Default, StyleFlags::BOLD);
        let flags: &[(&str, StyleFlags)] = &[
            ("Normal",        StyleFlags::empty()),
            ("Bold",          StyleFlags::BOLD),
            ("Dim",           StyleFlags::DIM),
            ("Italic",        StyleFlags::ITALIC),
            ("Underline",     StyleFlags::UNDERLINE),
            ("Reverse",       StyleFlags::REVERSE),
            ("Strikethrough", StyleFlags::STRIKETHROUGH),
        ];
        for (i, (name, flag)) in flags.iter().enumerate() {
            canvas.print(2 + i as u16 * 17, 9, &format!("{:<16}", name),
                Color::Basic(BasicColor::Cyan, true), Color::Default, *flag);
        }
    }

    fn render_256(&self, canvas: &mut SubCanvas<'_>, w: u16, h: u16) {
        // Draw the 6×6×6 color cube (indices 16–231) as a grid.
        canvas.print(0, 2, " xterm 6×6×6 color cube (indices 16–231):",
            Color::Default, Color::Default, StyleFlags::BOLD);

        let block = "  ";
        let mut row = 3u16;
        for r in 0u8..6 {
            for g in 0u8..6 {
                let col_start = g as u16 * 14;
                for b in 0u8..6 {
                    let idx = 16 + r * 36 + g * 6 + b;
                    canvas.print(
                        col_start + b as u16 * 2, row,
                        block,
                        Color::Ansi256(idx), Color::Ansi256(idx), StyleFlags::empty(),
                    );
                }
            }
            row += 1;
        }

        // Grayscale ramp (232–255)
        row += 1;
        canvas.print(0, row, " Grayscale ramp (232–255):", Color::Default, Color::Default, StyleFlags::BOLD);
        row += 1;
        for k in 0u8..24 {
            let idx = 232 + k;
            canvas.print(
                2 + k as u16 * 2, row,
                block,
                Color::Ansi256(idx), Color::Ansi256(idx), StyleFlags::empty(),
            );
        }

        let _ = (w, h);
    }

    fn render_truecolor(&self, canvas: &mut SubCanvas<'_>, w: u16, h: u16) {
        canvas.print(0, 2, " True Color 24-bit gradients:",
            Color::Default, Color::Default, StyleFlags::BOLD);

        // Horizontal hue sweep
        let grad_w = (w.saturating_sub(4)).min(120) as usize;
        for x in 0..grad_w {
            let hue = x as f32 / grad_w as f32 * 360.0;
            let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
            canvas.set(x as u16 + 2, 4, console_ui::canvas::Cell {
                ch: '█', fg: Color::TrueColor(r, g, b), bg: Color::Default,
                style: StyleFlags::empty(), wide_continuation: false,
            });
        }

        // Saturation gradient
        canvas.print(0, 6, " Saturation (red, full → grey):", Color::Default, Color::Default, StyleFlags::DIM);
        for x in 0..grad_w {
            let sat = 1.0 - x as f32 / grad_w as f32;
            let (r, g, b) = hsv_to_rgb(0.0, sat, 1.0);
            canvas.set(x as u16 + 2, 7, console_ui::canvas::Cell {
                ch: '█', fg: Color::TrueColor(r, g, b), bg: Color::Default,
                style: StyleFlags::empty(), wide_continuation: false,
            });
        }

        // Value gradient
        canvas.print(0, 9, " Value (blue, bright → black):", Color::Default, Color::Default, StyleFlags::DIM);
        for x in 0..grad_w {
            let val = 1.0 - x as f32 / grad_w as f32;
            let (r, g, b) = hsv_to_rgb(240.0, 1.0, val);
            canvas.set(x as u16 + 2, 10, console_ui::canvas::Cell {
                ch: '█', fg: Color::TrueColor(r, g, b), bg: Color::Default,
                style: StyleFlags::empty(), wide_continuation: false,
            });
        }

        let _ = h;
    }

    pub fn handle_key(&mut self, key: Key) -> Action {
        match key {
            Key::Escape | Key::Char('q') => Action::GoTo(super::ScreenId::MainMenu),
            Key::Left  => { self.page = self.page.saturating_sub(1); Action::Continue }
            Key::Right => { self.page = (self.page + 1).min(2);      Action::Continue }
            _ => Action::Continue,
        }
    }
}

/// Simple HSV → RGB conversion.  H in [0, 360), S and V in [0, 1].
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = if h < 60.0       { (c, x, 0.0) }
                       else if h < 120.0  { (x, c, 0.0) }
                       else if h < 180.0  { (0.0, c, x) }
                       else if h < 240.0  { (0.0, x, c) }
                       else if h < 300.0  { (x, 0.0, c) }
                       else               { (c, 0.0, x) };
    (
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}
