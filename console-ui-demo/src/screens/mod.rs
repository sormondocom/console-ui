pub mod border_demo;
pub mod color_palette;
pub mod layout_demo;
pub mod main_menu;
pub mod save_to_file;
pub mod table_demo;

/// Every screen returns one of these actions to the app loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Stay on the current screen (re-render next frame).
    Continue,
    /// Navigate to a different screen.
    GoTo(ScreenId),
    /// Exit the application.
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenId {
    MainMenu,
    BorderDemo,
    TableDemo,
    LayoutDemo,
    ColorPalette,
    SaveToFile,
}
