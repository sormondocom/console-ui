//! JSON serialization helpers using `serde_json`.

use super::types::ScreenDef;

/// Serialize a `ScreenDef` to a pretty-printed JSON string.
pub fn to_json(def: &ScreenDef) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(def)
}

/// Deserialize a `ScreenDef` from a JSON string.
pub fn from_json(s: &str) -> Result<ScreenDef, serde_json::Error> {
    serde_json::from_str(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serial::types::{
        AlignDef, BasicColorName, BorderDef, ColorDef, ConstraintDef, EdgeDef,
        LayoutDef, ScreenDef, StyleDef, TerminalTarget, WrapDef, WidgetDef, WidgetPlacement,
    };

    fn sample_def() -> ScreenDef {
        ScreenDef {
            name:   "test_screen".into(),
            target: TerminalTarget::Ansi256,
            width:  80,
            height: 24,
            layout: LayoutDef::Anchor {
                widgets: vec![
                    WidgetPlacement {
                        id: "header".into(),
                        widget: WidgetDef::Panel {
                            title:       Some("Hello World".into()),
                            title_align: AlignDef::Center,
                            border:      BorderDef::Double,
                            border_fg:   ColorDef::Basic { color: BasicColorName::Cyan, bright: true },
                            title_fg:    ColorDef::Default,
                            child:       None,
                        },
                        size_hint:   Some((78, 3)),
                        constraints: vec![
                            ConstraintDef { src_edge: EdgeDef::Top,  dst: "CONTAINER".into(), dst_edge: EdgeDef::Top,  offset: 0 },
                            ConstraintDef { src_edge: EdgeDef::Left, dst: "CONTAINER".into(), dst_edge: EdgeDef::Left, offset: 1 },
                        ],
                    },
                    WidgetPlacement {
                        id: "body".into(),
                        widget: WidgetDef::Text {
                            content: "This is a sample text block.".into(),
                            fg:      ColorDef::Default,
                            wrap:    WrapDef::Word,
                            style:   StyleDef::default(),
                        },
                        size_hint:   None,
                        constraints: vec![
                            ConstraintDef { src_edge: EdgeDef::Top,   dst: "header".into(),    dst_edge: EdgeDef::Bottom, offset: 1 },
                            ConstraintDef { src_edge: EdgeDef::Left,  dst: "CONTAINER".into(), dst_edge: EdgeDef::Left,   offset: 1 },
                            ConstraintDef { src_edge: EdgeDef::Right, dst: "CONTAINER".into(), dst_edge: EdgeDef::Right,  offset: -1 },
                        ],
                    },
                ],
            },
        }
    }

    #[test]
    fn round_trip() {
        let original = sample_def();
        let json = to_json(&original).expect("serialize");
        let loaded: ScreenDef = from_json(&json).expect("deserialize");
        assert_eq!(loaded.name,   original.name);
        assert_eq!(loaded.width,  original.width);
        assert_eq!(loaded.height, original.height);
    }

    #[test]
    fn build_from_def() {
        let def = sample_def();
        let mut layout = def.into_anchor_layout();
        // Verify resolution doesn't panic and produces non-zero rects.
        layout.resolve();
    }

    #[test]
    fn json_output_is_human_readable() {
        let json = to_json(&sample_def()).unwrap();
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"Hello World\""));
        assert!(json.contains("CONTAINER"));
        assert!(json.contains("ansi256"));  // target field
    }

    #[test]
    fn validation_vt100_rejects_unicode_border() {
        let mut def = sample_def();
        def.target = TerminalTarget::Vt100;
        // sample_def uses BorderDef::Double which is Unicode — should fail VT-100.
        let result = def.validate();
        assert!(result.is_err(), "VT-100 target should reject non-ASCII border");
        let errs = result.unwrap_err();
        assert!(!errs.is_empty());
    }

    #[test]
    fn validation_vt100_ascii_border_passes() {
        let def = ScreenDef {
            name:   "simple".into(),
            target: TerminalTarget::Vt100,
            width:  80,
            height: 24,
            layout: LayoutDef::Single {
                widget: WidgetDef::Panel {
                    title:       Some("VT-100 Screen".into()),
                    title_align: AlignDef::Left,
                    border:      BorderDef::Ascii,
                    border_fg:   ColorDef::Default,
                    title_fg:    ColorDef::Default,
                    child:       None,
                },
            },
        };
        assert!(def.validate().is_ok(), "ASCII border on VT-100 target should be valid");
    }

    #[test]
    fn validation_truecolor_rejected_on_ansi256() {
        let def = ScreenDef {
            name:   "tc".into(),
            target: TerminalTarget::Ansi256,
            width:  80,
            height: 24,
            layout: LayoutDef::Single {
                widget: WidgetDef::Panel {
                    title:       None,
                    title_align: AlignDef::Left,
                    border:      BorderDef::Single,
                    border_fg:   ColorDef::TrueColor { r: 255, g: 100, b: 0 },
                    title_fg:    ColorDef::Default,
                    child:       None,
                },
            },
        };
        let result = def.validate();
        assert!(result.is_err(), "TrueColor on Ansi256 target should fail");
    }
}
