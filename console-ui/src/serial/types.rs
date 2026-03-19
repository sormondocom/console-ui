//! Data types that describe a screen layout in a serializable form.
//!
//! These types mirror the live widget/layout types but are pure data —
//! no `Box<dyn Widget>`, no borrowed canvases.  They can be cheaply cloned,
//! diffed, stored, and transmitted.

#[cfg(feature = "serde-json")]
use serde::{Deserialize, Serialize};

use crate::border::BorderStyle;
use crate::color::{BasicColor, Color, StyleFlags};
use crate::layout::anchor::{AnchorId, AnchorLayout, Edge};
use crate::widget::{Align, Menu, Panel, Table, TextBlock, WrapMode};

// ---------------------------------------------------------------------------
// Color definition
// ---------------------------------------------------------------------------

/// Serializable color value.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-json", serde(tag = "type", rename_all = "snake_case"))]
pub enum ColorDef {
    Default,
    Basic  { color: BasicColorName, bright: bool },
    Ansi256 { index: u8 },
    TrueColor { r: u8, g: u8, b: u8 },
}

impl ColorDef {
    pub fn to_color(&self) -> Color {
        match self {
            ColorDef::Default              => Color::Default,
            ColorDef::Basic { color, bright } => Color::Basic(color.to_basic(), *bright),
            ColorDef::Ansi256 { index }    => Color::Ansi256(*index),
            ColorDef::TrueColor { r, g, b } => Color::TrueColor(*r, *g, *b),
        }
    }
}

impl Default for ColorDef {
    fn default() -> Self { ColorDef::Default }
}

impl From<Color> for ColorDef {
    fn from(c: Color) -> Self {
        match c {
            Color::Default               => ColorDef::Default,
            Color::Basic(bc, bright)     => ColorDef::Basic { color: bc.into(), bright },
            Color::Ansi256(i)            => ColorDef::Ansi256 { index: i },
            Color::TrueColor(r, g, b)    => ColorDef::TrueColor { r, g, b },
        }
    }
}

/// Serializable name for the 8 basic ANSI colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-json", serde(rename_all = "snake_case"))]
pub enum BasicColorName {
    Black, Red, Green, Yellow, Blue, Magenta, Cyan, White,
}

impl BasicColorName {
    pub fn to_basic(self) -> BasicColor {
        match self {
            BasicColorName::Black   => BasicColor::Black,
            BasicColorName::Red     => BasicColor::Red,
            BasicColorName::Green   => BasicColor::Green,
            BasicColorName::Yellow  => BasicColor::Yellow,
            BasicColorName::Blue    => BasicColor::Blue,
            BasicColorName::Magenta => BasicColor::Magenta,
            BasicColorName::Cyan    => BasicColor::Cyan,
            BasicColorName::White   => BasicColor::White,
        }
    }
}

impl From<BasicColor> for BasicColorName {
    fn from(c: BasicColor) -> Self {
        match c {
            BasicColor::Black   => BasicColorName::Black,
            BasicColor::Red     => BasicColorName::Red,
            BasicColor::Green   => BasicColorName::Green,
            BasicColor::Yellow  => BasicColorName::Yellow,
            BasicColor::Blue    => BasicColorName::Blue,
            BasicColor::Magenta => BasicColorName::Magenta,
            BasicColor::Cyan    => BasicColorName::Cyan,
            BasicColor::White   => BasicColorName::White,
        }
    }
}

// ---------------------------------------------------------------------------
// Style flags definition
// ---------------------------------------------------------------------------

/// Serializable list of style attribute names.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
pub struct StyleDef {
    #[cfg_attr(feature = "serde-json", serde(default))] pub bold:          bool,
    #[cfg_attr(feature = "serde-json", serde(default))] pub dim:           bool,
    #[cfg_attr(feature = "serde-json", serde(default))] pub italic:        bool,
    #[cfg_attr(feature = "serde-json", serde(default))] pub underline:     bool,
    #[cfg_attr(feature = "serde-json", serde(default))] pub blink:         bool,
    #[cfg_attr(feature = "serde-json", serde(default))] pub reverse:       bool,
    #[cfg_attr(feature = "serde-json", serde(default))] pub strikethrough: bool,
}

impl StyleDef {
    pub fn to_flags(&self) -> StyleFlags {
        let mut f = StyleFlags::empty();
        if self.bold          { f |= StyleFlags::BOLD; }
        if self.dim           { f |= StyleFlags::DIM; }
        if self.italic        { f |= StyleFlags::ITALIC; }
        if self.underline     { f |= StyleFlags::UNDERLINE; }
        if self.blink         { f |= StyleFlags::BLINK; }
        if self.reverse       { f |= StyleFlags::REVERSE; }
        if self.strikethrough { f |= StyleFlags::STRIKETHROUGH; }
        f
    }
}

impl From<StyleFlags> for StyleDef {
    fn from(f: StyleFlags) -> Self {
        Self {
            bold:          f.contains(StyleFlags::BOLD),
            dim:           f.contains(StyleFlags::DIM),
            italic:        f.contains(StyleFlags::ITALIC),
            underline:     f.contains(StyleFlags::UNDERLINE),
            blink:         f.contains(StyleFlags::BLINK),
            reverse:       f.contains(StyleFlags::REVERSE),
            strikethrough: f.contains(StyleFlags::STRIKETHROUGH),
        }
    }
}

// ---------------------------------------------------------------------------
// Border definition
// ---------------------------------------------------------------------------

/// Serializable border style name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-json", serde(rename_all = "snake_case"))]
pub enum BorderDef {
    Ascii, #[default] Single, Double, Rounded, Heavy,
}

impl BorderDef {
    pub fn to_border_style(self) -> BorderStyle {
        match self {
            BorderDef::Ascii   => BorderStyle::Ascii,
            BorderDef::Single  => BorderStyle::Single,
            BorderDef::Double  => BorderStyle::Double,
            BorderDef::Rounded => BorderStyle::Rounded,
            BorderDef::Heavy   => BorderStyle::Heavy,
        }
    }
}

impl From<BorderStyle> for BorderDef {
    fn from(s: BorderStyle) -> Self {
        match s {
            BorderStyle::Ascii   => BorderDef::Ascii,
            BorderStyle::Single  => BorderDef::Single,
            BorderStyle::Double  => BorderDef::Double,
            BorderStyle::Rounded => BorderDef::Rounded,
            BorderStyle::Heavy   => BorderDef::Heavy,
        }
    }
}

// ---------------------------------------------------------------------------
// Alignment definition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-json", serde(rename_all = "snake_case"))]
pub enum AlignDef { #[default] Left, Center, Right }

impl AlignDef {
    pub fn to_align(self) -> Align {
        match self { AlignDef::Left => Align::Left, AlignDef::Center => Align::Center, AlignDef::Right => Align::Right }
    }
}

// ---------------------------------------------------------------------------
// Widget definitions
// ---------------------------------------------------------------------------

/// A serializable description of a single widget.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-json", serde(tag = "kind", rename_all = "snake_case"))]
pub enum WidgetDef {
    /// Bordered panel with optional title.
    Panel {
        #[cfg_attr(feature = "serde-json", serde(default))]
        title:       Option<String>,
        #[cfg_attr(feature = "serde-json", serde(default))]
        title_align: AlignDef,
        #[cfg_attr(feature = "serde-json", serde(default))]
        border:      BorderDef,
        #[cfg_attr(feature = "serde-json", serde(default))]
        border_fg:   ColorDef,
        #[cfg_attr(feature = "serde-json", serde(default))]
        title_fg:    ColorDef,
        /// Nested child widget (optional).
        #[cfg_attr(feature = "serde-json", serde(default))]
        child:       Option<Box<WidgetDef>>,
    },

    /// Table with headers and data rows.
    Table {
        #[cfg_attr(feature = "serde-json", serde(default))]
        headers:     Option<Vec<String>>,
        #[cfg_attr(feature = "serde-json", serde(default))]
        rows:        Vec<Vec<String>>,
        #[cfg_attr(feature = "serde-json", serde(default))]
        col_align:   Vec<AlignDef>,
        #[cfg_attr(feature = "serde-json", serde(default))]
        border:      BorderDef,
        #[cfg_attr(feature = "serde-json", serde(default))]
        border_fg:   ColorDef,
        #[cfg_attr(feature = "serde-json", serde(default))]
        header_fg:   ColorDef,
        #[cfg_attr(feature = "serde-json", serde(default))]
        alt_row_bg:  Option<ColorDef>,
    },

    /// Selectable item list.
    Menu {
        items:       Vec<String>,
        #[cfg_attr(feature = "serde-json", serde(default))]
        cursor_fg:   ColorDef,
        #[cfg_attr(feature = "serde-json", serde(default))]
        cursor_bg:   ColorDef,
    },

    /// Word-wrapped text block.
    Text {
        content: String,
        #[cfg_attr(feature = "serde-json", serde(default))]
        fg:      ColorDef,
        #[cfg_attr(feature = "serde-json", serde(default))]
        wrap:    WrapDef,
        #[cfg_attr(feature = "serde-json", serde(default))]
        style:   StyleDef,
    },

    /// Empty placeholder (renders nothing).
    Empty,
}

impl WidgetDef {
    /// Instantiate a live `Box<dyn Widget>` from this definition.
    pub fn build(self) -> Box<dyn crate::widget::Widget> {
        match self {
            WidgetDef::Panel { title, title_align, border, border_fg, title_fg, child } => {
                let mut p = Panel::new()
                    .border_style(border.to_border_style())
                    .border_fg(border_fg.to_color())
                    .title_fg(title_fg.to_color())
                    .title_align(title_align.to_align());
                if let Some(t) = title { p = p.title(t); }
                if let Some(c) = child { p = p.child(c.build()); }
                Box::new(p)
            }
            WidgetDef::Table { headers, rows, col_align, border, border_fg, header_fg, alt_row_bg } => {
                let mut t = Table::new()
                    .border_style(border.to_border_style())
                    .border_fg(border_fg.to_color())
                    .col_align(col_align.iter().map(|a| a.to_align()).collect())
                    .rows(rows.iter().map(|r| r.iter().map(|c| c.as_str()).collect::<Vec<_>>()).collect::<Vec<_>>());
                if let Some(h) = headers { t = t.headers(h.iter().map(|s| s.as_str()).collect::<Vec<_>>()); }
                t.header_fg  = header_fg.to_color();
                if let Some(ab) = alt_row_bg { t.alt_row_bg = Some(ab.to_color()); }
                Box::new(t)
            }
            WidgetDef::Menu { items, cursor_fg, cursor_bg } => {
                let mut m = Menu::new(items);
                m.cursor_fg = cursor_fg.to_color();
                m.cursor_bg = cursor_bg.to_color();
                Box::new(m)
            }
            WidgetDef::Text { content, fg, wrap, style } => {
                let tb = TextBlock::new(content)
                    .fg(fg.to_color())
                    .wrap(wrap.to_wrap_mode())
                    .style(style.to_flags());
                Box::new(tb)
            }
            WidgetDef::Empty => {
                Box::new(crate::widget::panel::Panel::new())
            }
        }
    }
}

/// Serializable wrap mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-json", serde(rename_all = "snake_case"))]
pub enum WrapDef { #[default] Word, Char, None }

impl WrapDef {
    pub fn to_wrap_mode(self) -> WrapMode {
        match self { WrapDef::Word => WrapMode::Word, WrapDef::Char => WrapMode::Char, WrapDef::None => WrapMode::None }
    }
}

// ---------------------------------------------------------------------------
// Constraint / anchor placement definitions
// ---------------------------------------------------------------------------

/// Serializable edge name for anchor constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-json", serde(rename_all = "snake_case"))]
pub enum EdgeDef { Top, Right, Bottom, Left }

impl EdgeDef {
    pub fn to_edge(self) -> Edge {
        match self { EdgeDef::Top => Edge::Top, EdgeDef::Right => Edge::Right, EdgeDef::Bottom => Edge::Bottom, EdgeDef::Left => Edge::Left }
    }
}

/// One constraint declaration in the serialized form.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
pub struct ConstraintDef {
    pub src_edge: EdgeDef,
    /// ID string of the destination widget.  `"CONTAINER"` refers to the layout box.
    pub dst:      String,
    pub dst_edge: EdgeDef,
    #[cfg_attr(feature = "serde-json", serde(default))]
    pub offset:   i16,
}

/// A widget with its anchor placement constraints.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
pub struct WidgetPlacement {
    /// Stable string ID for this widget (used in `ConstraintDef.dst`).
    pub id:          String,
    pub widget:      WidgetDef,
    /// Override preferred size.  If absent, the widget's own `preferred_size()` is used.
    #[cfg_attr(feature = "serde-json", serde(default))]
    pub size_hint:   Option<(u16, u16)>,
    #[cfg_attr(feature = "serde-json", serde(default))]
    pub constraints: Vec<ConstraintDef>,
}

// ---------------------------------------------------------------------------
// Layout definitions
// ---------------------------------------------------------------------------

/// The top-level layout strategy for a screen.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-json", serde(tag = "strategy", rename_all = "snake_case"))]
pub enum LayoutDef {
    /// Anchor-relative layout.  Widgets positioned by edge constraints.
    Anchor { widgets: Vec<WidgetPlacement> },

    /// Horizontal split.
    HSplit { ratio: f32, first: Box<ScreenDef>, second: Box<ScreenDef> },

    /// Vertical split.
    VSplit { ratio: f32, first: Box<ScreenDef>, second: Box<ScreenDef> },

    /// 2×2 grid.
    Grid4 {
        h_ratio: f32, v_ratio: f32,
        top_left:     Box<ScreenDef>,
        top_right:    Box<ScreenDef>,
        bottom_left:  Box<ScreenDef>,
        bottom_right: Box<ScreenDef>,
    },

    /// A single widget fills the entire region.
    Single { widget: WidgetDef },
}

// ---------------------------------------------------------------------------
// Terminal target — declares what class of terminal this screen is built for
// ---------------------------------------------------------------------------

/// The terminal class this screen was designed for.
///
/// Choosing a target enforces constraints during the interactive builder
/// (e.g. a VT-100 screen can't use Unicode borders or 256-color palettes)
/// and lets the runtime check compatibility before rendering.
///
/// Targets are ordered: `Vt100 < Vt220 < Ansi256 < TrueColor`.  A screen
/// designed for a lower target will always render correctly on a higher-
/// capability terminal, but not necessarily the other way around.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-json", serde(rename_all = "snake_case"))]
pub enum TerminalTarget {
    /// VT-100 / plain ANSI: 8 fg + 8 bg colors, ASCII box-drawing only,
    /// no Unicode.  Widest compatibility — works on legacy hardware,
    /// serial terminals, and `TERM=dumb` environments.
    Vt100,

    /// VT-220 / modern ANSI: adds Unicode box-drawing characters (requires
    /// UTF-8 locale or code page 65001), still limited to 8+8 ANSI colors.
    #[default]
    Vt220,

    /// xterm 256-color: Unicode box-drawing + 256-color `ESC[38;5;Nm`
    /// sequences.  Supported by virtually all modern terminal emulators.
    Ansi256,

    /// 24-bit True Color: full RGB via `ESC[38;2;R;G;Bm`.  Requires a
    /// modern emulator (iTerm2, Windows Terminal, most Linux terminals
    /// built after 2016).
    TrueColor,
}

impl TerminalTarget {
    /// Returns `true` if this screen can be rendered on a terminal with
    /// `actual_level` capability without any incompatible features.
    pub fn is_compatible_with(self, actual: crate::term::ColorLevel) -> bool {
        use crate::term::ColorLevel;
        match self {
            TerminalTarget::Vt100     => true, // always compatible
            TerminalTarget::Vt220     => actual >= ColorLevel::Vt100Basic || actual == ColorLevel::None,
            TerminalTarget::Ansi256   => actual >= ColorLevel::Ansi256,
            TerminalTarget::TrueColor => actual >= ColorLevel::TrueColor,
        }
    }

    /// The `ColorLevel` this target requires at minimum.
    pub fn required_color_level(self) -> crate::term::ColorLevel {
        use crate::term::ColorLevel;
        match self {
            TerminalTarget::Vt100     => ColorLevel::None,
            TerminalTarget::Vt220     => ColorLevel::Vt100Basic,
            TerminalTarget::Ansi256   => ColorLevel::Ansi256,
            TerminalTarget::TrueColor => ColorLevel::TrueColor,
        }
    }

    /// Whether this target allows Unicode box-drawing characters.
    pub fn unicode_allowed(self) -> bool {
        matches!(self, TerminalTarget::Vt220 | TerminalTarget::Ansi256 | TerminalTarget::TrueColor)
    }

    /// Human-readable description shown in the builder.
    pub fn description(self) -> &'static str {
        match self {
            TerminalTarget::Vt100     => "VT-100  — 8 ANSI colors, ASCII borders only",
            TerminalTarget::Vt220     => "VT-220  — 8 ANSI colors + Unicode box-drawing",
            TerminalTarget::Ansi256   => "256-color — xterm 256-color palette + Unicode",
            TerminalTarget::TrueColor => "True Color — 24-bit RGB + Unicode (modern terminals)",
        }
    }

    pub fn all() -> &'static [TerminalTarget] {
        &[
            TerminalTarget::Vt100,
            TerminalTarget::Vt220,
            TerminalTarget::Ansi256,
            TerminalTarget::TrueColor,
        ]
    }

    /// Validate a `ColorDef` against this target.  Returns an error string
    /// if the color exceeds what the target supports.
    pub fn validate_color(self, color: &ColorDef, field: &str) -> Option<String> {
        match (self, color) {
            (TerminalTarget::Vt100 | TerminalTarget::Vt220, ColorDef::Ansi256 { .. }) =>
                Some(format!("{}: Ansi256 color not supported on {} target", field, self.description())),
            (TerminalTarget::Vt100 | TerminalTarget::Vt220, ColorDef::TrueColor { .. }) =>
                Some(format!("{}: TrueColor not supported on {} target", field, self.description())),
            (TerminalTarget::Ansi256, ColorDef::TrueColor { .. }) =>
                Some(format!("{}: TrueColor not supported on Ansi256 target", field)),
            _ => None,
        }
    }

    /// Validate a `BorderDef` against this target.
    pub fn validate_border(self, border: &BorderDef) -> Option<String> {
        if !self.unicode_allowed() && *border != BorderDef::Ascii {
            Some(format!(
                "border style '{:?}' requires Unicode — use Ascii for VT-100 target", border
            ))
        } else {
            None
        }
    }
}

/// A list of validation errors returned by `ScreenDef::validate()`.
#[derive(Debug, Clone)]
pub struct ValidationErrors(pub Vec<String>);

impl ValidationErrors {
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn errors(&self) -> &[String] { &self.0 }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.0 { writeln!(f, "  • {}", e)?; }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Screen definition — the top-level unit of serialization
// ---------------------------------------------------------------------------

/// A complete, serializable screen definition.
///
/// Designed to be stored in `.json` files, embedded with `include_str!`, or
/// built programmatically and then serialized for later use.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-json", derive(Serialize, Deserialize))]
pub struct ScreenDef {
    /// Human-readable name for this screen (used in builder menus).
    pub name:   String,
    /// The terminal class this layout targets.  The builder enforces that
    /// only features available on this class are used.
    pub target: TerminalTarget,
    /// Expected terminal width.  Layouts are re-flowed at runtime if the
    /// actual terminal size differs.
    pub width:  u16,
    /// Expected terminal height.
    pub height: u16,
    pub layout: LayoutDef,
}

impl ScreenDef {
    /// Validate that all widget colors, borders, and features are compatible
    /// with `self.target`.  Returns `Ok(())` if valid, `Err(ValidationErrors)`
    /// with a list of human-readable problems otherwise.
    ///
    /// The interactive builder calls this after every change to give live
    /// feedback; downstream applications can call it at load time to catch
    /// mismatched layouts before rendering.
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errs = Vec::new();
        Self::validate_layout(&self.layout, self.target, &mut errs);
        if errs.is_empty() { Ok(()) } else { Err(ValidationErrors(errs)) }
    }

    fn validate_layout(layout: &LayoutDef, target: TerminalTarget, errs: &mut Vec<String>) {
        match layout {
            LayoutDef::Anchor { widgets } => {
                for w in widgets {
                    Self::validate_widget_def(&w.widget, target, &w.id, errs);
                }
            }
            LayoutDef::Single { widget } => {
                Self::validate_widget_def(widget, target, "root", errs);
            }
            LayoutDef::HSplit { first, second, .. } |
            LayoutDef::VSplit { first, second, .. } => {
                Self::validate_layout(&first.layout, target, errs);
                Self::validate_layout(&second.layout, target, errs);
            }
            LayoutDef::Grid4 { top_left, top_right, bottom_left, bottom_right, .. } => {
                for sub in [top_left, top_right, bottom_left, bottom_right] {
                    Self::validate_layout(&sub.layout, target, errs);
                }
            }
        }
    }

    fn validate_widget_def(widget: &WidgetDef, target: TerminalTarget, id: &str, errs: &mut Vec<String>) {
        match widget {
            WidgetDef::Panel { border, border_fg, title_fg, child, .. } => {
                if let Some(e) = target.validate_border(border) { errs.push(format!("[{}] {}", id, e)); }
                if let Some(e) = target.validate_color(border_fg, "border_fg") { errs.push(format!("[{}] {}", id, e)); }
                if let Some(e) = target.validate_color(title_fg,  "title_fg")  { errs.push(format!("[{}] {}", id, e)); }
                if let Some(child) = child { Self::validate_widget_def(child, target, id, errs); }
            }
            WidgetDef::Table { border, border_fg, header_fg, alt_row_bg, .. } => {
                if let Some(e) = target.validate_border(border) { errs.push(format!("[{}] {}", id, e)); }
                if let Some(e) = target.validate_color(border_fg, "border_fg") { errs.push(format!("[{}] {}", id, e)); }
                if let Some(e) = target.validate_color(header_fg, "header_fg") { errs.push(format!("[{}] {}", id, e)); }
                if let Some(ab) = alt_row_bg {
                    if let Some(e) = target.validate_color(ab, "alt_row_bg") { errs.push(format!("[{}] {}", id, e)); }
                }
            }
            WidgetDef::Menu { cursor_fg, cursor_bg, .. } => {
                if let Some(e) = target.validate_color(cursor_fg, "cursor_fg") { errs.push(format!("[{}] {}", id, e)); }
                if let Some(e) = target.validate_color(cursor_bg, "cursor_bg") { errs.push(format!("[{}] {}", id, e)); }
            }
            WidgetDef::Text { fg, .. } => {
                if let Some(e) = target.validate_color(fg, "fg") { errs.push(format!("[{}] {}", id, e)); }
            }
            WidgetDef::Empty => {}
        }
    }

    /// Build a live `AnchorLayout` from this definition.
    ///
    /// If the layout strategy is `Anchor`, the constraints are wired up.
    /// Other strategies are flattened to a single-panel anchor layout.
    pub fn into_anchor_layout(self) -> AnchorLayout {
        let mut layout = AnchorLayout::new(self.width, self.height);
        Self::populate_anchor(&mut layout, self.layout);
        layout
    }

    fn populate_anchor(layout: &mut AnchorLayout, def: LayoutDef) {
        match def {
            LayoutDef::Anchor { widgets } => {
                // Two-pass: first add all widgets (collecting string→AnchorId map),
                // then wire constraints.
                let mut id_map: std::collections::HashMap<String, AnchorId> = std::collections::HashMap::new();
                let mut placements: Vec<(AnchorId, Vec<ConstraintDef>)> = Vec::new();

                for placement in widgets {
                    let widget = placement.widget.build();
                    let aid    = layout.add(widget, placement.size_hint);
                    id_map.insert(placement.id, aid);
                    placements.push((aid, placement.constraints));
                }

                // Wire constraints.
                for (src_id, constraints) in placements {
                    for c in constraints {
                        let dst_id = if c.dst == "CONTAINER" {
                            AnchorLayout::CONTAINER
                        } else {
                            match id_map.get(&c.dst) {
                                Some(&aid) => aid,
                                None => {
                                    eprintln!("console-ui: unknown constraint target '{}', skipping", c.dst);
                                    continue;
                                }
                            }
                        };
                        layout.constrain(src_id, c.src_edge.to_edge(), dst_id, c.dst_edge.to_edge(), c.offset);
                    }
                }
            }
            LayoutDef::Single { widget } => {
                let w = widget.build();
                let id = layout.add(w, None);
                layout.fill(id, 0);
            }
            // For split layouts, recursively flatten — each sub-def becomes
            // a panel-wrapped anchor.  Production code would render these as
            // proper split panes; this gives a working fallback.
            LayoutDef::HSplit { first, second, .. } |
            LayoutDef::VSplit { first, second, .. } => {
                Self::populate_anchor(layout, first.layout);
                Self::populate_anchor(layout, second.layout);
            }
            LayoutDef::Grid4 { top_left, top_right, bottom_left, bottom_right, .. } => {
                for sub in [top_left, top_right, bottom_left, bottom_right] {
                    Self::populate_anchor(layout, sub.layout);
                }
            }
        }
    }
}
