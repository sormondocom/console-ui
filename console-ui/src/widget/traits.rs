use crate::canvas::SubCanvas;
use crate::event::Key;

/// Every UI widget implements this trait.
///
/// Widgets render into a `SubCanvas` whose dimensions equal the region
/// allocated to them by the layout manager.  Widgets must not write outside
/// the provided canvas.
pub trait Widget {
    fn render(&self, canvas: &mut SubCanvas<'_>);

    /// Minimum `(width, height)` for the widget to be usable.
    fn min_size(&self) -> (u16, u16);

    /// Preferred `(width, height)` when unconstrained.
    fn preferred_size(&self) -> (u16, u16) { self.min_size() }
}

/// A widget that can receive keyboard input.
///
/// Returns `true` if the key was consumed, `false` to propagate it up.
pub trait InteractiveWidget: Widget {
    fn handle_key(&mut self, key: Key) -> bool;
    fn is_focused(&self) -> bool;
    fn set_focused(&mut self, focused: bool);
}
