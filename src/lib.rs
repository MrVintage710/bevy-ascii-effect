mod render;
mod ui;
mod ascii;

pub mod prelude {
    pub use crate::ascii::*;
    pub use crate::ui::bounds::AsciiNode;
    pub use crate::ui::component::button::AsciiButton;
    pub use crate::ui::position::AsciiPosition;
    pub use crate::ui::HorizontalAlignment;
    pub use crate::ui::VerticalAlignment;
    pub use crate::ui::command::AsciiUiCommandExtention;
    pub use crate::ui::AsciiUi;
    pub use crate::ui::util::Value;
    pub use crate::ui::component::AsciiComponent;
    pub use crate::ui::bounds::AsciiBounds;
    pub use crate::ui::util::Variable;
    pub use crate::ui::util::break_string_into_lines;
    pub use crate::ui::util::AsciiComponentHoverExitedEvent;
    pub use crate::ui::util::AsciiComponentHoverEnteredEvent;
    pub use crate::ui::util::AsciiComponentButtonClicked as AsciiComponentButtonClickedEvent;
    pub use crate::ui::buffer::AsciiBuffer;
    pub use crate::ui::buffer::AsciiBoxDrawer;
    pub use crate::ui::buffer::AsciiTextDrawer;
    pub use crate::ui::component::AsciiComponentPlugin;
    pub use crate::ui::AsciiMarkDirtyEvent;
    pub use crate::ui::BorderType;
    pub use crate::ui::character::Color as AsciiColor;
    pub use crate::ui::character::AsciiCharacter;
    pub use crate::ui::character::Character;
}