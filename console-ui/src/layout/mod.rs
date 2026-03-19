mod region;
pub mod anchor;
pub mod split2;
pub mod split3;
pub mod split4;

pub use region::Rect;
pub use anchor::{AnchorId, AnchorLayout, Constraint, Edge};
pub use split2::{HSplit, Pane2, VSplit};
pub use split3::Split3;
pub use split4::{Pane4, Split4};
