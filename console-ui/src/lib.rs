//! # console-ui
//!
//! A cross-platform console UI library supporting terminals from VT-100 (8-color,
//! ASCII borders) through ANSI 256-color to True Color 24-bit.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use console_ui::{
//!     term::{init_caps, RawModeGuard},
//!     canvas::{Canvas, Renderer},
//!     widget::{Panel, Widget},
//!     border::BorderStyle,
//!     color::Color,
//! };
//!
//! fn main() -> crossterm::Result<()> {
//!     let caps = init_caps();
//!     let _raw = RawModeGuard::enter()?;
//!     let mut renderer = Renderer::new(caps);
//!     let mut canvas = Canvas::new(caps.cols, caps.rows);
//!
//!     let panel = Panel::new()
//!         .title("Hello, console-ui!")
//!         .border_style(BorderStyle::Rounded);
//!
//!     let mut root = canvas.sub(0, 0, caps.cols, caps.rows);
//!     panel.render(&mut root);
//!
//!     renderer.render(&canvas)?;
//!
//!     // Wait for a keypress.
//!     console_ui::event::read_key()?;
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! Layers (each only depends on those below it):
//!
//! 1. **`term`** — capability detection (`TermCaps`), raw mode RAII guard.
//! 2. **`color`** — `Color` enum (Basic / Ansi256 / TrueColor) with graceful
//!    downgrade, `StyleFlags` bitfield.
//! 3. **`border`** — `BorderStyle` enum and `BorderGlyphs` lookup tables.
//! 4. **`canvas`** — Off-screen `Canvas` + `SubCanvas` cell buffer; `Renderer`
//!    double-buffer diff flush.
//! 5. **`widget`** — `Widget` / `InteractiveWidget` traits; `Panel`, `Table`,
//!    `Menu`, `TextBlock` implementations.
//! 6. **`layout`** — `HSplit`, `VSplit`, `Split3`, `Split4` layout managers.
//! 7. **`event`** — `Key` abstraction over crossterm input events.

pub mod border;
pub mod canvas;
pub mod color;
pub mod event;
pub mod layout;
pub mod term;
pub mod widget;

/// Re-export of the `crossterm::Result` type used throughout the crate.
pub use std::io::Result;

/// Convenience prelude — `use console_ui::prelude::*` for the most common items.
pub mod prelude {
    pub use crate::border::BorderStyle;
    pub use crate::canvas::{Canvas, Renderer, SubCanvas};
    pub use crate::color::{BasicColor, Color, StyleFlags};
    pub use crate::event::{poll_key, read_key, Key};
    pub use crate::layout::{HSplit, Pane2, Pane4, Split3, Split4, VSplit};
    pub use crate::term::{caps, init_caps, ColorLevel, RawModeGuard, TermCaps};
    pub use crate::widget::{Align, InteractiveWidget, Menu, Panel, Table, TextBlock, Widget, Wr