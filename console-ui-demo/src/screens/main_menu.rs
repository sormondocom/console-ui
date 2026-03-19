use console_ui::prelude::*;
use super::{Action, ScreenId};

const ITEMS: &[&str] = &[
    "Border Style Preview",
    "Table Demo",
    "Layout Demo (2 / 3 / 4 Panes)",
    "Color Palette",
    "Save Current View to File",
    "",
    "Quit",
];

const LOGO: &str = r#"
   _____                      _       _    _ _
  / ____|                    | |     | |  | |_|
 | |     ___  _ __  ___  ___ | | ___ | |  | |_
 | |    / _ \| '_ \/ __|/ _ \| |/ _ \| |  | | |
 | |___| (_) | | | \__ \ (_) | |  __/| |__| | |
  \_____\___/|_| |_|___/\___/|_|\___| \____/|_|
"#;

pub struct MainMenu {
    menu: Menu,
}

impl MainMenu {
    pub fn new() -> Self {
        let mut menu = Menu::new(ITEMS.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        menu.cursor_style = StyleFlags::REVERSE | StyleFlags::BOLD;
        Self { menu }
    }

    pub fn render(&self, canvas: &mut Canvas) {
        let w = canvas.width();
        let h = canvas.height();

        let mut root = canvas.sub(0, 0, w, h);

        // Outer panel
        let panel = Panel::new()
            .title(" console-ui demo ")
            .title_align(Align::Center)
            .border_style(BorderStyle::Double)
            .border_fg(Color::Basic(BasicColor::Cyan, true));
        panel.render(&mut root);

        // Logo (inner area starts at row 1, col 1)
        let inner_w = w.saturating_sub(2);
        let inner_h = h.saturating_sub(2);
        let mut inner = root.sub(1, 1, inner_w, inner_h);

        let logo_lines: Vec<&str> = LOGO.lines().filter(|l| !l.is_empty()).collect();
        for (i, line) in logo_lines.iter().enumerate() {
            if i as u16 >= inner_h { break; }
            inner.print(
                0, i as u16, line,
                Color::Basic(BasicColor::Cyan, false),
                Color::Default,
                StyleFlags::empty(),
            );
        }

        let logo_h = logo_lines.len() as u16 + 1;

        // Help line
        let help = " Arrow keys: navigate   Enter: select   q: quit ";
        if logo_h + 1 < inner_h {
            inner.print(
                1, logo_h,
                help,
                Color::Basic(BasicColor::White, false),
                Color::Default,
                StyleFlags::DIM,
            );
        }

        // Menu
        let menu_y = logo_h + 2;
        let menu_h = inner_h.saturating_sub(menu_y);
        if menu_h > 0 {
            let mut menu_area = inner.sub(2, menu_y, inner_w.saturating_sub(4), menu_h);
            self.menu.render(&mut menu_area);
        }
    }

    pub fn handle_key(&mut self, key: Key) -> Action {
        match key {
            Key::Char('q') | Key::Escape => return Action::Quit,
            Key::Enter => {
                return match self.menu.selected() {
                    0 => Action::GoTo(ScreenId::BorderDemo),
                    1 => Action::GoTo(ScreenId::TableDemo),
                    2 => Action::GoTo(ScreenId::LayoutDemo),
                    3 => Action::GoTo(ScreenId::ColorPalette),
                    4 => Action::GoTo(ScreenId::SaveToFile),
                    6 => Action::Quit,
                    _ => Action::Continue,
                };
            }
            _ => { self.menu.handle_key(key); }
        }
        Action::Continue
    }
}
