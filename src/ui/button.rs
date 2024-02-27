use bevy::{ecs::{component::Component, system::Query}, input::mouse::MouseButton};

use super::{buffer::AsciiBuffer, character::Color, component::AsciiComponent, BorderType, HorizontalAlignment, VerticalAlignment};

#[derive(Component)]
pub struct AsciiButton {
    bg_color: Color,
    hover_color: Color,
    is_hovering: bool,
    button_text: String,
}

impl AsciiButton {
    pub fn from_string(text : &str) -> Self {
        AsciiButton {
            bg_color : Color::Black,
            hover_color : Color::Grey,
            is_hovering : false,
            button_text : text.to_string(),
        }
    }
}

impl AsciiComponent for AsciiButton {
    type UpdateQuery = (
        
    );

    fn render(&self, buffer : &mut AsciiBuffer) {
        buffer.square().border(BorderType::Full).draw();
    }
}

