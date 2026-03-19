# console-ui

A cross-platform terminal UI library for Rust — built to look great everywhere, from a 1980s VT-100 to a modern True Color terminal.

[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-support%20this%20project-yellow?logo=buy-me-a-coffee)](https://buymeacoffee.com/sormondocom)

---

## Overview

**console-ui** is a layered TUI library that handles the hard parts of terminal UI: capability detection, color downgrade, double-buffer rendering, Unicode-aware layout, and a serializable screen definition format that lets you ship UI layouts as data.

It was originally built as the UI layer for **[console-pgp-chat](https://github.com/sormondocom/console-pgp-chat)** — a fully console-based, end-to-end encrypted PGP chat application. That project's requirement to run reliably on anything from a modern True Color terminal down to a legacy VT-100 is what shaped console-ui's core design philosophy.

---

## Terminal Compatibility

One of the core design goals is **graceful degradation** — the library detects what the terminal actually supports and scales down cleanly rather than breaking.

| Feature | VT-100 / Dumb | ANSI / xterm | 256-Color | True Color |
|---|---|---|---|---|
| Text rendering | ✓ | ✓ | ✓ | ✓ |
| Basic 8 ANSI colors | — | ✓ | ✓ | ✓ |
| 256-color palette | — | — | ✓ | ✓ |
| 24-bit RGB color | — | — | — | ✓ |
| Unicode box-drawing | — | ✓ | ✓ | ✓ |
| ASCII fallback borders | ✓ | ✓ | ✓ | ✓ |

### Color Downgrade Chain

When a color is used that the terminal can't display, the library automatically converts it to the nearest supported color rather than emitting garbage or falling back to defaults blindly:

```
TrueColor (24-bit RGB)
    ↓  nearest match in xterm 6×6×6 cube + grayscale ramp
Ansi256 (256-palette index)
    ↓  nearest euclidean distance in RGB space
Basic (8 ANSI colors, with bright variants)
    ↓  drop all color
Default (terminal default fg/bg)
```

The same principle applies to borders: `BorderStyle::with_caps()` automatically demotes Unicode box-drawing characters to ASCII `+-|` when the terminal reports no Unicode support.

### Capability Detection

At startup, `init_caps()` inspects the environment and returns a `TermCaps` struct:

```rust
pub struct TermCaps {
    pub color_level: ColorLevel,  // None, Vt100Basic, Ansi256, TrueColor
    pub unicode: bool,
    pub cols: u16,
    pub rows: u16,
}
```

Detection checks (in order): `$NO_COLOR`, `$TERM`, `$COLORTERM`, `$WT_SESSION` (Windows Terminal), `$TERM_PROGRAM` (iTerm2, VSCode, Hyper, Apple Terminal), and known xterm/rxvt families. Unknown terminals get a safe conservative baseline.

---

## Architecture

The library is organized in strict dependency layers — each layer only depends on those below it:

```
serial   ← (optional) JSON screen definitions
event    ← Key abstraction over crossterm input
layout   ← AnchorLayout, HSplit, VSplit, Split4
widget   ← Panel, Table, Menu, TextBlock
canvas   ← Canvas, SubCanvas, Renderer (double-buffer)
border   ← BorderStyle, BorderGlyphs
color    ← Color, StyleFlags
term     ← TermCaps detection, RawModeGuard
```

---

## Serializable Screen Definitions

One of console-ui's more distinctive features is a **JSON-based screen definition format** that lets you describe entire UI layouts as data. This makes it possible to:

- Ship UI layouts embedded in your binary via `include_str!()`
- Load layouts from config files or over the network at runtime
- Share screen definitions between apps that use the library
- Validate a layout against a target terminal class before deploying

### The Format

A `ScreenDef` is a self-contained description of a screen: its name, the minimum terminal class it requires, its dimensions, and a layout definition tree.

```json
{
  "name": "main_panel",
  "target": "TrueColor",
  "cols": 80,
  "rows": 24,
  "layout": {
    "Anchor": {
      "widgets": [
        {
          "id": "sidebar",
          "widget": {
            "Panel": {
              "title": "Navigation",
              "title_align": "Left",
              "border_style": "Rounded",
              "fg": { "TrueColor": [200, 220, 255] },
              "bg": "Default"
            }
          }
        },
        {
          "id": "content",
          "widget": {
            "Table": {
              "headers": ["Name", "Value", "Status"],
              "rows": []
            }
          }
        }
      ],
      "constraints": [
        { "widget": "sidebar", "edge": "Left",   "target": "CONTAINER", "target_edge": "Left",  "offset": 1 },
        { "widget": "sidebar", "edge": "Top",    "target": "CONTAINER", "target_edge": "Top",   "offset": 1 },
        { "widget": "sidebar", "edge": "Bottom", "target": "CONTAINER", "target_edge": "Bottom","offset": -1 },
        { "widget": "sidebar", "edge": "Right",  "target": "sidebar",   "target_edge": "Left",  "offset": 24 },
        { "widget": "content", "edge": "Left",   "target": "sidebar",   "target_edge": "Right", "offset": 1 },
        { "widget": "content", "edge": "Right",  "target": "CONTAINER", "target_edge": "Right", "offset": -1 },
        { "widget": "content", "edge": "Top",    "target": "CONTAINER", "target_edge": "Top",   "offset": 1 },
        { "widget": "content", "edge": "Bottom", "target": "CONTAINER", "target_edge": "Bottom","offset": -1 }
      ]
    }
  }
}
```

### Validation

`ScreenDef::validate()` checks the entire definition against its declared `TerminalTarget` before instantiation — catching issues like a VT-100 layout that accidentally uses a 256-color palette index or a Unicode rounded border:

```rust
let def: ScreenDef = from_json(json_str)?;
def.validate()?;  // returns Err(ValidationErrors) if incompatible features are used
let layout = def.into_anchor_layout();
```

### Embedding Layouts

```rust
let json = include_str!("screens/main_panel.json");
let layout = from_json(json)?.into_anchor_layout();
```

### Layout Types

| Type | Description |
|---|---|
| `Anchor` | Constraint-based positioning — attach widget edges to other widget edges with pixel offsets |
| `HSplit` | Horizontal split with a ratio-adjustable divider |
| `VSplit` | Vertical split with a ratio-adjustable divider |
| `Grid4` | 2×2 fixed grid |
| `Single` | Full-screen single widget |

---

## Widgets

| Widget | Description |
|---|---|
| `Panel` | Bordered box with optional title (left / center / right aligned) |
| `Table` | Column-aligned rows with optional headers and alternating row colors |
| `Menu` | Selectable item list with configurable cursor highlight |
| `TextBlock` | Word-wrapped text with alignment and style flags |

All widgets implement the `Widget` trait (`render`, `min_size`, `preferred_size`). Interactive widgets additionally implement `InteractiveWidget` (`handle_key`, `is_focused`, `set_focused`).

---

## Border Styles

Five border styles are available, automatically demoted to ASCII when the terminal doesn't support Unicode:

| Style | Example |
|---|---|
| `Ascii` | `+-|` |
| `Single` | `┌─┐ │ └─┘` |
| `Double` | `╔═╗ ║ ╚═╝` |
| `Rounded` | `╭─╮ │ ╰─╯` |
| `Heavy` | `┏━┓ ┃ ┗━┛` |

---

## Rendering

The `Renderer` uses **double-buffering** — it keeps a copy of what was last drawn and only emits escape sequences for cells that actually changed. This keeps output minimal and prevents flickering, even on slow connections or legacy terminals.

Wide characters (CJK, emoji) are handled correctly: a wide glyph occupies two columns, and the renderer tracks the continuation cell to avoid cursor misalignment.

---

## Feature Flags

| Feature | Default | Description |
|---|---|---|
| `color` | on | 256-color palette and downgrade tables (~3 KB lookup tables) |
| `serde-json` | on | JSON serialization of screen definitions |

To build a minimal binary targeting only VT-100:

```toml
console-ui = { version = "0.1", default-features = false }
```

---

## Running the Demo

```sh
cargo run -p console-ui-demo
```

The demo showcases all major features interactively:

- **Border Demo** — cycle through all five border styles
- **Table Demo** — column-aligned data with headers
- **Layout Demo** — anchor constraint positioning
- **Color Palette** — 256-color or True Color explorer
- **Save to File** — export the current canvas to plain text (no escape codes)

---

## Support

If console-ui saves you time or you just think terminal UIs deserve more love, consider buying me a coffee:

[![Buy Me a Coffee](https://img.shields.io/badge/☕%20Buy%20Me%20a%20Coffee-sormondocom-yellow)](https://buymeacoffee.com/sormondocom)

---

## License

MIT
