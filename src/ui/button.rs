use bevy::{ecs::system::Query, input::mouse::MouseButton};

use super::{buffer::{AsciiBounds, AsciiBuffer}, character::Color, node::AsciiUiComponent, AsciiUiNode, BorderType, HorizontalAlignment, VerticalAlignment};

pub struct AsciiButton {
    bg_color: Color,
    hover_color: Color,
    is_hovering: bool,
    button_text: String,
}

impl AsciiButton {
    pub const ID : &'static str = "BUTTON";
    
    pub fn from_string(text : &str) -> Self {
        AsciiButton {
            bg_color : Color::Black,
            hover_color : Color::Grey,
            is_hovering : false,
            button_text : text.to_string(),
        }
    }
}

impl AsciiUiComponent for AsciiButton {
    fn name(&self) -> &str {
        Self::ID
    }
    
    fn render(&self, buffer: &AsciiBuffer) {
        
        let square = buffer.square();
        
        let square = if self.is_hovering {
            square.bg_color(self.hover_color)
        } else {
            square.bg_color(self.bg_color)
        };
        
        let inner = square
            .border(BorderType::Full)
            .draw();
        
        if let Some(inner) = inner {
            inner.text(self.button_text.as_str())
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center)
                .draw();
        }
    }
}
