use bevy::input::mouse::MouseButton;

use super::{buffer::{AsciiBounds, AsciiBuffer}, character::Color, node::AsciiUiComponent, AsciiUiNode, HorizontalAlignment, VerticalAlignment};

pub struct AsciiButton;

impl AsciiUiComponent for AsciiButton {
    fn name(&self) -> &str {
        "Button"
    }
    
    fn render(&self, buffer: &AsciiBuffer) {
        println!("Rendering button");
        buffer.square().bg_color(Color::Red).draw();
    }
}
