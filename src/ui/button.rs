use bevy::input::mouse::MouseButton;

use super::{buffer::AsciiBounds, AsciiUiNode, Color, HorizontalAlignment, VerticalAlignment};

pub struct AsciiButton {
    pub text: String,
    hover_color : Color,
    base_color : Color,
    bg_color : Color,
    border_color : Color,
    text_color : Color,
    bounds : AsciiBounds,
}

impl AsciiButton {
    pub fn new(text: &str) -> Self {
        AsciiButton {
            text: text.to_string(),
            hover_color: Color::Grey,
            base_color: Color::White,
            bg_color: Color::Black,
            border_color: Color::White,
            text_color: Color::White,
            bounds: AsciiBounds::default(),
        }
    }
}

impl AsciiUiNode for AsciiButton {
    fn render(&mut self, buffer: &super::buffer::AsciiBuffer) {
        let inner = buffer.square()
            .border(super::BorderType::Full)
            .bg_color(self.bg_color)
            .border_color(self.border_color)
            .draw()
        ;
        
        if let Some(inner) = inner {
            inner.text(&self.text)
                .text_color(self.text_color)
                .bg_color(self.bg_color)
                .vertical_alignment(VerticalAlignment::Center)
                .horizontal_alignment(HorizontalAlignment::Center)
                .draw()
            ;
        }
        
        self.bounds = buffer.bounds().clone();
    }

    fn update(&mut self, context: &mut super::AsciiUiContext) {
        if let Some((cursor_x, cursor_y)) = context.cursor_pos() {
            if self.bounds.is_within(cursor_x, cursor_y) {
                if self.bg_color != self.hover_color {
                    context.mark_dirty()
                }
                self.border_color = self.hover_color;
                
                if context.cursor_input().just_pressed(MouseButton::Left) {
                    
                }
            } else {
                if self.bg_color != self.base_color {
                    context.mark_dirty()
                }
                self.border_color = self.base_color;
            }
        }
    }
}
