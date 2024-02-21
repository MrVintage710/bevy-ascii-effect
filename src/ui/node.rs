use bevy::prelude::*;

use super::{buffer::{AsciiBounds, AsciiBuffer}, HorizontalAlignment, VerticalAlignment};

//=============================================================================
//             AsciiUiComponent
//=============================================================================

pub trait AsciiUiComponent {
    fn name(&self) -> &str;
    
    fn render(&self, buffer: &AsciiBuffer);
}

//=============================================================================
//             AsciiUiLayout
//=============================================================================

#[derive(Clone)]
pub enum AsciiUiLayout {
    Absolute(AsciiBounds),
    Relative(AsciiBounds),
    Align(u32, u32, HorizontalAlignment, VerticalAlignment),
    VerticalSlice(u32, u32),
    HorizontalSlice(u32, u32),
}

//=============================================================================
//             AsciiNode
//=============================================================================

#[derive(Component)]
pub struct AsciiUiNode {
    pub(crate) bounds : AsciiBounds,
    pub(crate) component : Box<dyn AsciiUiComponent + Send + Sync>,
    pub(crate) layout : AsciiUiLayout,
    pub(crate) hidden : bool,
    pub(crate) is_dirty : bool,
}

impl AsciiUiNode {
    pub fn bounds(&self) -> &AsciiBounds {
        &self.bounds
    }
    
    pub fn hide(&mut self) {
        if self.hidden == false {
            self.hidden = true;
            self.is_dirty = true;
        }
    }
    
    pub fn show(&mut self) {
        if self.hidden == true {
            self.hidden = false;
            self.is_dirty = true;
        }
    }
    
    pub fn is_of_type(&self, type_name: &str) -> bool {
        self.component.name() == type_name
    }
    
    pub fn layout(&self) -> &AsciiUiLayout {
        &self.layout
    }
    
    pub fn set_layout(&mut self, layout : AsciiUiLayout) {
        self.layout = layout;
        self.is_dirty = true;
    }
    
    pub fn render(&self, buffer : &AsciiBuffer) {
        self.component.render(buffer);
    }
    
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }
}




