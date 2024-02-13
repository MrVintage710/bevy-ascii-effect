use bevy::prelude::*;

use super::buffer::{AsciiBounds, AsciiBuffer};

//=============================================================================
//             AsciiUiComponent
//=============================================================================

pub trait AsciiUiComponent {
    fn render(&self, buffer: &AsciiBuffer);
}

//=============================================================================
//             AsciiNode
//=============================================================================

#[derive(Component)]
pub struct AsciiUiNode {
    bounds : AsciiBounds,
    component : Box<dyn AsciiUiComponent + Send + Sync>,
    pub hidden : bool,
}

impl AsciiUiNode {
    pub fn bounds(&self) -> &AsciiBounds {
        &self.bounds
    }
    
    pub fn render(&self, buffer : &AsciiBuffer) {
        self.component.render(buffer);
    }
}




