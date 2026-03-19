/// A rectangle in terminal coordinates (zero-based, col/row).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x:      u16,
    pub y:      u16,
    pub width:  u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }

    /// Inset the rectangle by `margin` on all sides.
    pub fn inner(self, margin: u16) -> Self {
        let m2 = margin * 2;
        Self {
            x:      self.x + margin,
            y:      self.y + margin,
            width:  self.width.saturating_sub(m2),
            height: self.height.saturating_sub(m2),
        }
    }

    /// Split horizontally at `ratio` (0.0 = all left, 1.0 = all right).
    /// Returns `(left, right)`.
    pub fn split_h(self, ratio: f32) -> (Self, Self) {
        let left_w = ((self.width as f32 * ratio) as u16).clamp(0, self.width);
        let right_w = self.width.saturating_sub(left_w);
        (
            Self::new(self.x, self.y, left_w, self.height),
            Self::new(self.x + left_w, self.y, right_w, self.height),
        )
    }

    /// Split vertically at `ratio` (0.0 = all top, 1.0 = all bottom).
    /// Returns `(top, bottom)`.
    pub fn split_v(self, ratio: f32) -> (Self, Self) {
        let top_h = ((self.height as f32 * ratio) as u16).clamp(0, self.height);
        let bot_h = self.height.saturating_sub(top_h);
        (
            Self::new(self.x, self.y, self.width, top_h),
            Self::new(self.x, self.y + top_h, self.width, bot_h),
        )
    }
}
