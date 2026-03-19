use console_ui::prelude::*;
use super::Action;

pub struct TableDemo {
    scroll: usize,
}

impl TableDemo {
    pub fn new() -> Self { Self { scroll: 0 } }

    pub fn render(&self, canvas: &mut Canvas) {
        let w = canvas.width();
        let h = canvas.height();
        let mut root = canvas.sub(0, 0, w, h);

        let outer = Panel::new()
            .title(" Table Demo ")
            .title_align(Align::Center)
            .border_style(BorderStyle::Heavy)
            .border_fg(Color::Basic(BasicColor::Green, true));
        outer.render(&mut root);

        let inner_w = w.saturating_sub(2);
        let inner_h = h.saturating_sub(2);
        let mut inner = root.sub(1, 1, inner_w, inner_h);

        // Help line
        inner.print(0, 0,
            " ↑ / ↓ scroll   Esc: back ",
            Color::Basic(BasicColor::White, false), Color::Default, StyleFlags::DIM);

        // Sample table
        let mut table = Table::new()
            .headers(vec!["ID", "Name", "Role", "Department", "Status"])
            .rows(vec![
                vec!["001", "Alice Johnson",   "Senior Engineer",  "Platform",     "Active"],
                vec!["002", "Bob Smith",        "Product Manager",  "Product",      "Active"],
                vec!["003", "Carol White",      "UX Designer",      "Design",       "On Leave"],
                vec!["004", "David Brown",      "DevOps Engineer",  "Infra",        "Active"],
                vec!["005", "Eve Martinez",     "Data Scientist",   "Analytics",    "Active"],
                vec!["006", "Frank Lee",        "QA Engineer",      "Quality",      "Inactive"],
                vec!["007", "Grace Kim",        "Tech Lead",        "Platform",     "Active"],
                vec!["008", "Hank Turner",      "Backend Engineer", "Platform",     "Active"],
                vec!["009", "Iris Chen",        "Frontend Engineer","Web",          "Active"],
                vec!["010", "Jack Wilson",      "Security Analyst", "Security",     "Active"],
                vec!["011", "Karen Davis",      "Scrum Master",     "Delivery",     "Active"],
                vec!["012", "Liam Garcia",      "ML Engineer",      "Analytics",    "Active"],
            ])
            .col_align(vec![Align::Center, Align::Left, Align::Left, Align::Left, Align::Center])
            .border_style(BorderStyle::Single)
            .border_fg(Color::Basic(BasicColor::Cyan, false));

        table.scroll_row = self.scroll;
        table.alt_row_bg = Some(Color::Ansi256(235));
        table.header_fg  = Color::Basic(BasicColor::Yellow, true);
        table.header_style = StyleFlags::BOLD | StyleFlags::UNDERLINE;

        let table_h = inner_h.saturating_sub(2);
        let mut table_area = inner.sub(0, 2, inner_w, table_h);
        table.render(&mut table_area);
    }

    pub fn handle_key(&mut self, key: Key) -> Action {
        match key {
            Key::Escape | Key::Char('q') => Action::GoTo(super::ScreenId::MainMenu),
            Key::Up   => { self.scroll = self.scroll.saturating_sub(1); Action::Continue }
            Key::Down => { self.scroll += 1; Action::Continue }
            _ => Action::Continue,
        }
    }
}
