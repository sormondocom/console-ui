mod screens;

use console_ui::prelude::*;
use screens::{
    Action, ScreenId,
    border_demo::BorderDemo,
    color_palette::ColorPalette,
    layout_demo::LayoutDemo,
    main_menu::MainMenu,
    save_to_file::SaveToFile,
    table_demo::TableDemo,
};

enum Screen {
    MainMenu(MainMenu),
    BorderDemo(BorderDemo),
    TableDemo(TableDemo),
    LayoutDemo(LayoutDemo),
    ColorPalette(ColorPalette),
    SaveToFile(SaveToFile),
}

impl Screen {
    fn render(&mut self, canvas: &mut Canvas) {
        canvas.clear();
        match self {
            Screen::MainMenu(s)    => s.render(canvas),
            Screen::BorderDemo(s)  => s.render(canvas),
            Screen::TableDemo(s)   => s.render(canvas),
            Screen::LayoutDemo(s)  => s.render(canvas),
            Screen::ColorPalette(s) => s.render(canvas),
            Screen::SaveToFile(s)  => s.render(canvas),
        }
    }

    fn handle_key(&mut self, key: Key, canvas: &Canvas) -> Action {
        match self {
            Screen::MainMenu(s)    => s.handle_key(key),
            Screen::BorderDemo(s)  => s.handle_key(key),
            Screen::TableDemo(s)   => s.handle_key(key),
            Screen::LayoutDemo(s)  => s.handle_key(key),
            Screen::ColorPalette(s) => s.handle_key(key),
            Screen::SaveToFile(s)  => s.handle_key(key, canvas),
        }
    }
}

fn make_screen(id: ScreenId) -> Screen {
    match id {
        ScreenId::MainMenu    => Screen::MainMenu(MainMenu::new()),
        ScreenId::BorderDemo  => Screen::BorderDemo(BorderDemo::new()),
        ScreenId::TableDemo   => Screen::TableDemo(TableDemo::new()),
        ScreenId::LayoutDemo  => Screen::LayoutDemo(LayoutDemo::new()),
        ScreenId::ColorPalette => Screen::ColorPalette(ColorPalette::new()),
        ScreenId::SaveToFile  => Screen::SaveToFile(SaveToFile::new()),
    }
}

fn main() -> std::io::Result<()> {
    let caps = init_caps();
    let _raw = RawModeGuard::enter()?;

    let mut renderer = Renderer::new(caps);
    let mut canvas   = Canvas::new(caps.cols, caps.rows);
    let mut screen   = Screen::MainMenu(MainMenu::new());

    renderer.clear_screen()?;

    loop {
        screen.render(&mut canvas);
        renderer.render(&canvas)?;

        let key = read_key()?;

        // Handle terminal resize before passing to screen.
        if let Key::Resize(cols, rows) = key {
            canvas.resize(cols, rows);
            renderer.force_render(&canvas)?;
            continue;
        }

        match screen.handle_key(key, &canvas) {
            Action::Continue        => {}
            Action::Quit            => break,
            Action::GoTo(id)        => {
                screen = make_screen(id);
                renderer.clear_screen()?;
            }
        }
    }

    Ok(())
}
