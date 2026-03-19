use crate::canvas::SubCanvas;
use crate::widget::traits::Widget;

/// Three-pane layout variants.
///
/// Each variant stores the three child widgets and a ratio controlling the
/// primary split point.
pub enum Split3 {
    /// Top row split into two, bottom row spans full width.
    /// ```text
    /// ┌────┬────┐
    /// │ A  │ B  │
    /// ├────┴────┤
    /// │    C    │
    /// └─────────┘
    /// ```
    TopTwoBottomOne {
        top_a:   Box<dyn Widget>,
        top_b:   Box<dyn Widget>,
        bottom:  Box<dyn Widget>,
        h_ratio: f32,  // left/right split within the top row
        v_ratio: f32,  // top row vs bottom row
    },

    /// Top row spans full width, bottom row split into two.
    /// ```text
    /// ┌─────────┐
    /// │    A    │
    /// ├────┬────┤
    /// │ B  │ C  │
    /// └────┴────┘
    /// ```
    TopOneBottomTwo {
        top:     Box<dyn Widget>,
        bottom_a: Box<dyn Widget>,
        bottom_b: Box<dyn Widget>,
        h_ratio: f32,
        v_ratio: f32,
    },

    /// Left pane spans full height, right side split top/bottom.
    /// ```text
    /// ┌────┬────┐
    /// │    │ B  │
    /// │ A  ├────┤
    /// │    │ C  │
    /// └────┴────┘
    /// ```
    LeftOneRightTwo {
        left:    Box<dyn Widget>,
        right_a: Box<dyn Widget>,
        right_b: Box<dyn Widget>,
        h_ratio: f32,
        v_ratio: f32,
    },

    /// Left side split top/bottom, right pane spans full height.
    /// ```text
    /// ┌────┬────┐
    /// │ A  │    │
    /// ├────┤ C  │
    /// │ B  │    │
    /// └────┴────┘
    /// ```
    LeftTwoRightOne {
        left_a:  Box<dyn Widget>,
        left_b:  Box<dyn Widget>,
        right:   Box<dyn Widget>,
        h_ratio: f32,
        v_ratio: f32,
    },
}

impl Widget for Split3 {
    fn render(&self, canvas: &mut SubCanvas<'_>) {
        let w = canvas.width();
        let h = canvas.height();

        match self {
            Split3::TopTwoBottomOne { top_a, top_b, bottom, h_ratio, v_ratio } => {
                let top_h  = ((h as f32 * v_ratio) as u16).clamp(2, h.saturating_sub(2));
                let bot_h  = h.saturating_sub(top_h);
                let left_w = ((w as f32 * h_ratio) as u16).clamp(2, w.saturating_sub(2));
                let right_w = w.saturating_sub(left_w);
                {
                    let mut sub = canvas.sub(0, 0, left_w, top_h);
                    top_a.render(&mut sub);
                }
                {
                    let mut sub = canvas.sub(left_w, 0, right_w, top_h);
                    top_b.render(&mut sub);
                }
                {
                    let mut sub = canvas.sub(0, top_h, w, bot_h);
                    bottom.render(&mut sub);
                }
            }
            Split3::TopOneBottomTwo { top, bottom_a, bottom_b, h_ratio, v_ratio } => {
                let top_h  = ((h as f32 * v_ratio) as u16).clamp(2, h.saturating_sub(2));
                let bot_h  = h.saturating_sub(top_h);
                let left_w = ((w as f32 * h_ratio) as u16).clamp(2, w.saturating_sub(2));
                let right_w = w.saturating_sub(left_w);
                {
                    let mut sub = canvas.sub(0, 0, w, top_h);
                    top.render(&mut sub);
                }
                {
                    let mut sub = canvas.sub(0, top_h, left_w, bot_h);
                    bottom_a.render(&mut sub);
                }
                {
                    let mut sub = canvas.sub(left_w, top_h, right_w, bot_h);
                    bottom_b.render(&mut sub);
                }
            }
            Split3::LeftOneRightTwo { left, right_a, right_b, h_ratio, v_ratio } => {
                let left_w  = ((w as f32 * h_ratio) as u16).clamp(2, w.saturating_sub(2));
                let right_w = w.saturating_sub(left_w);
                let top_h   = ((h as f32 * v_ratio) as u16).clamp(2, h.saturating_sub(2));
                let bot_h   = h.saturating_sub(top_h);
                {
                    let mut sub = canvas.sub(0, 0, left_w, h);
                    left.render(&mut sub);
                }
                {
                    let mut sub = canvas.sub(left_w, 0, right_w, top_h);
                    right_a.render(&mut sub);
                }
                {
                    let mut sub = canvas.sub(left_w, top_h, right_w, bot_h);
                    right_b.render(&mut sub);
                }
            }
            Split3::LeftTwoRightOne { left_a, left_b, right, h_ratio, v_ratio } => {
                let left_w  = ((w as f32 * h_ratio) as u16).clamp(2, w.saturating_sub(2));
                let right_w = w.saturating_sub(left_w);
                let top_h   = ((h as f32 * v_ratio) as u16).clamp(2, h.saturating_sub(2));
                let bot_h   = h.saturating_sub(top_h);
                {
                    let mut sub = canvas.sub(0, 0, left_w, top_h);
                    left_a.render(&mut sub);
                }
                {
                    let mut sub = canvas.sub(0, top_h, left_w, bot_h);
                    left_b.render(&mut sub);
                }
                {
                    let mut sub = canvas.sub(left_w, 0, right_w, h);
                    right.render(&mut sub);
                }
            }
        }
    }

    fn min_size(&self) -> (u16, u16) { (20, 10) }
}
