mod render;
mod ui;
mod ascii;

pub mod prelude {
    pub use crate::ascii::*;
    pub use crate::ui::bounds::AsciiGlobalBounds;
    pub use crate::ui::button::AsciiButton;
    pub use crate::ui::position::AsciiPosition;
    pub use crate::ui::HorizontalAlignment;
    pub use crate::ui::VerticalAlignment;
}