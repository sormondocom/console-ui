//! # Screen Definition Serialization
//!
//! Serialize and deserialize complete screen layouts as JSON (or any serde
//! format) so that screens designed in the interactive builder can be:
//!
//! - Saved to `.json` files
//! - Embedded in other Rust applications as `include_str!("my_screen.json")`
//! - Instantiated at runtime with [`ScreenDef::into_layout`]
//!
//! ## Round-trip example
//!
//! ```rust,ignore
//! use console_ui::serial::{ScreenDef, WidgetDef, LayoutDef, ColorDef, BorderDef};
//!
//! // Build a definition programmatically (or load from JSON).
//! let def = ScreenDef {
//!     name: "dashboard".into(),
//!     width: 80, height: 24,
//!     layout: LayoutDef::Anchor {
//!         widgets: vec![
//!             WidgetPlacement {
//!                 id: "title".into(),
//!                 widget: WidgetDef::Panel {
//!                     title: Some("My App".into()),
//!                     border: BorderDef::Double,
//!                     border_fg: ColorDef::Basic { name: "cyan", bright: true },
//!                     ..Default::default()
//!                 },
//!                 size_hint: Some((40, 3)),
//!                 constraints: vec![
//!                     ConstraintDef { src_edge: "top",  dst: "CONTAINER", dst_edge: "top",  offset: 0 },
//!                     ConstraintDef { src_edge: "left", dst: "CONTAINER", dst_edge: "left", offset: 0 },
//!                 ],
//!             },
//!         ],
//!     },
//! };
//!
//! // Serialize.
//! let json = def.to_json().unwrap();
//! std::fs::write("dashboard.json", &json).unwrap();
//!
//! // Deserialize and build a live layout.
//! let loaded = ScreenDef::from_json(&json).unwrap();
//! let mut anchor = loaded.into_anchor_layout();
//! ```

#[cfg(feature = "serde-json")]
mod json_impl;

mod types;

pub use types::*;

#[cfg(feature = "serde-json")]
pub use json_impl::{from_json, to_json};
