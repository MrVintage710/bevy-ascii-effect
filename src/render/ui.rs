use bevy::prelude::*;

pub struct AsciiUi {
    nodes: Vec<Box<dyn AsciiUiNode>>,
    buffer: AsciiBuffer,
}

pub struct AsciiBuffer {
    data: Vec<AsciiCharacter>,
    width: u32,
    height: u32,
    is_dirty: bool,
}

pub enum AsciiCharacter {
    Set {
        index: u16,
        text_color: u8,
        background_color: u8,
    },
    Unset,
}

pub trait AsciiUiNode {
    fn render(&self, buffer: &mut AsciiBuffer) -> AsciiBuffer;

    fn is_dirty(&self) -> bool;
}
