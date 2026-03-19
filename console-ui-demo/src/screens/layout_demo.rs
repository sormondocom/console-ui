use console_ui::prelude::*;
use console_ui::canvas::SubCanvas;
use super::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode { Two, Three, Four }

pub struct LayoutDemo {
    pub mode:       LayoutMode,
    pub h_ratio:    f32,
    pub v_ratio:    f32,
    pub resize_mode: bool,
}

impl LayoutDemo {
    pub fn new() -> Self {
        Self { mode: LayoutMode::Four, h_ratio: 0.5, v_ratio: 0.5, resize_mode: false }
    }

    pub fn render(&self, canvas: &mut Canvas) {
        let w = canvas.width();
        let h = canvas.height();
        let mut root = canvas.sub(0, 0, w, h);

        // Status bar at bottom
        let status = if self.resize_mode {
            " RESIZE MODE: arrow keys move divider   r / Esc: exit resize   2/3/4: change layout "
        } else {
            " 2: 2-pane   3: 3-pane   4: 4-pane   r: resize mode   s: save   Esc: back "
        };
        let status_row = h.saturating_sub(1);
        root.print(
            0, status_row, status,
            Color::Basic(BasicColor::Black, false),
            Color::Basic(BasicColor::Cyan, false),
            StyleFlags::empty(),
        );

        let layout_h = h.saturating_sub(1);

        match self.mode {
            LayoutMode::Two => self.render_two(&mut root, w, layout_h),
            LayoutMode::Three => self.render_three(&mut root, w, layout_h),
            LayoutMode::Four => self.render_four(&mut root, w, layout_h),
        }
    }

    fn make_pane(label: &str, border: BorderStyle, fg: Color) -> Panel {
        Panel::new()
            .title(format!(" {} ", label))
            .title_align(Align::Center)
            .border_style(border)
            .border_fg(fg)
    }

    fn render_two(&self, canvas: &mut SubCanvas<'_>, w: u16, h: u16) {
        let div = ((w as f32 * self.h_ratio) as u16).clamp(4, w.saturating_sub(4));
        let div_color = if self.resize_mode {
            Color::Basic(BasicColor::Yellow, true)
        } else {
            Color::Default
        };

        let left  = Self::make_pane("Pane A", BorderStyle::Single, Color::Basic(BasicColor::Cyan,  true));
        let right = Self::make_pane("Pane B", BorderStyle::Single, Color::Basic(BasicColor::Green, true));

        { let mut s = canvas.sub(0, 0, div, h); left.render(&mut s); }
        canvas.fill_v(div, 0, h, '│', div_color, Color::Default);
        { let mut s = canvas.sub(div + 1, 0, w.saturating_sub(div + 1), h); right.render(&mut s); }

        self.render_resize_hint(canvas, div, h / 2);
    }

    fn render_three(&self, canvas: &mut SubCanvas<'_>, w: u16, h: u16) {
        let top_h = ((h as f32 * self.v_ratio) as u16).clamp(3, h.saturating_sub(3));
        let div   = ((w as f32 * self.h_ratio) as u16).clamp(4, w.saturating_sub(4));
        let bot_h = h.saturating_sub(top_h);

        let a = Self::make_pane("Pane A", BorderStyle::Single, Color::Basic(BasicColor::Cyan,    true));
        let b = Self::make_pane("Pane B", BorderStyle::Single, Color::Basic(BasicColor::Green,   true));
        let c = Self::make_pane("Pane C", BorderStyle::Single, Color::Basic(BasicColor::Magenta, true));

        { let mut s = canvas.sub(0, 0, div, top_h);       a.render(&mut s); }
        { let mut s = canvas.sub(div, 0, w - div, top_h); b.render(&mut s); }
        { let mut s = canvas.sub(0, top_h, w, bot_h);     c.render(&mut s); }
    }

    fn render_four(&self, canvas: &mut SubCanvas<'_>, w: u16, h: u16) {
        let div_col = ((w as f32 * self.h_ratio) as u16).clamp(4, w.saturating_sub(4));
        let div_row = ((h as f32 * self.v_ratio) as u16).clamp(3, h.saturating_sub(3));
        let div_color = if self.resize_mode {
            Color::Basic(BasicColor::Yellow, true)
        } else {
            Color::Default
        };

        let right_w  = w.saturating_sub(div_col + 1);
        let bottom_h = h.saturating_sub(div_row + 1);

        let colors = [
            Color::Basic(BasicColor::Cyan,    true),
            Color::Basic(BasicColor::Green,   true),
            Color::Basic(BasicColor::Magenta, true),
            Color::Basic(BasicColor::Yellow,  true),
        ];
        let labels = ["Top-Left", "Top-Right", "Bottom-Left", "Bottom-Right"];
        let regions = [
            (0u16,        0u16,     div_col, div_row),
            (div_col + 1, 0,        right_w, div_row),
            (0,           div_row + 1, div_col, bottom_h),
            (div_col + 1, div_row + 1, right_w, bottom_h),
        ];

        for (i, (x, y, rw, rh)) in regions.iter().enumerate() {
            let p = Self::make_pane(labels[i], BorderStyle::Rounded, colors[i]);
            let mut s = canvas.sub(*x, *y, *rw, *rh);
            p.render(&mut s);
        }

        canvas.fill_v(div_col, 0, h, '│', div_color, Color::Default);
        canvas.fill_h(0, div_row, w, '─', div_color, Color::Default);
        canvas.set(div_col, div_row, console_ui::canvas::Cell {
            ch: '┼', fg: div_color, bg: Color::Default,
            style: StyleFlags::empty(), wide_continuation: false,
        });

        self.render_resize_hint(canvas, div_col, div_row);
    }

    fn render_resize_hint(&self, canvas: &mut SubCanvas<'_>, col: u16, row: u16) {
        if self.resize_mode {
            canvas.set(col, row, console_ui::canvas::Cell {
                ch: '◆',
                fg: Color::Basic(BasicColor::Yellow, true),
                bg: Color::Default,
                style: StyleFlags::BOLD,
                wide_continuation: false,
            });
        }
    }

    pub fn handle_key(&mut self, key: Key) -> Action {
        const STEP: f32 = 0.05;

        if self.resize_mode {
            match key {
                Key::Left  => { self.h_ratio = (self.h_ratio - STEP).max(0.1); return Action::Continue; }
                Key::Right => { self.h_ratio = (self.h_ratio + STEP).min(0.9); return Action::Continue; }
                Key::Up    => { self.v_ratio = (self.v_ratio - STEP).max(0.1); return Action::Continue; }
                Key::Down  => { self.v_ratio = (self.v_ratio + STEP).min(0.9); return Action::Continue; }
                Key::Char('r') | Key::Escape => { self.resize_mode = false; return Action::Continue; }
                _ => {}
            }
        }

        match key {
            Key::Char('2') => { self.mode = LayoutMode::Two; Action::Continue }
            Key::Char('3') => { self.mode = LayoutMode::Three; Action::Continue }
            Key::Char('4') => { self.mode = LayoutMode::Four; Action::Continue }
            Key::Char('r') => { self.resize_mode = true; Action::Continue }
            Key::Char('s') => Action::GoTo(super::ScreenId::SaveToFile),
            Key::Escape | Key::Char('q') => Action::GoTo(super::ScreenId::MainMenu),
            _ => Action::Continue,
        }
    }
}
