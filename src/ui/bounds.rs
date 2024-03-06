use bevy::{prelude::*, utils::HashMap};

use crate::ascii::AsciiCamera;

use super::{position::AsciiPosition, util::Value, HorizontalAlignment, VerticalAlignment};

//=============================================================================
//             Plugin and Systems
//=============================================================================

pub struct AsciiBoundsPlugin;

impl Plugin for AsciiBoundsPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<AsciiBounds>()
            .register_type::<AsciiGlobalBounds>()
        ;
    }
}

//=============================================================================
//             Ascii Buffer
//=============================================================================

#[derive(Clone, Default, Debug, Reflect, PartialEq, Eq)]
pub struct AsciiBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub layer: u32,
}

impl AsciiBounds {
    pub fn new(x: i32, y: i32, width: u32, height: u32, layer: u32) -> Self {
        AsciiBounds {
            x,
            y,
            width,
            height,
            layer,
        }
    }

    pub fn from_dims(width: u32, height: u32) -> Self {
        AsciiBounds {
            x: 0,
            y: 0,
            width,
            height,
            layer: 0,
        }
    }

    pub fn with_layer(mut self, layer: u32) -> Self {
        self.layer = layer;
        self
    }

    pub fn is_within(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x <= self.x + self.width as i32
            && y >= self.y
            && y <= self.y + self.height as i32
    }

    pub fn is_within_local(&self, x: i32, y: i32) -> bool {
        let x = self.x + x;
        let y = self.y + y;
        x >= self.x
            && x <= self.x + self.width as i32
            && y >= self.y
            && y <= self.y + self.height as i32
    }

    pub fn relative(&self, child: &AsciiBounds) -> AsciiBounds {
        AsciiBounds {
            x: self.x + child.x,
            y: self.y + child.y,
            width: child.width,
            height: child.height,
            layer: child.layer + self.layer + 1,
        }
    }

    pub fn aligned(
        &self,
        width: impl Into<Value>,
        height: impl Into<Value>,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> AsciiBounds {
        AsciiPosition::create_bounds_aligned(
            width,
            height,
            horizontal_alignment,
            vertical_alignment,
            self,
        )
    }
}

//=============================================================================
//             Ascii Global Bounds
//=============================================================================

#[derive(Clone, Default, Debug, Reflect, Component)]
pub struct AsciiGlobalBounds {
    pub bounds: AsciiBounds,
    pub is_dirty: bool,
    pub clip_bounds: bool,
}

impl AsciiGlobalBounds {
    pub fn new(x: i32, y: i32, width: u32, height: u32, layer : u32) -> AsciiGlobalBounds {
        AsciiGlobalBounds {
            bounds: AsciiBounds::new(x, y, width, height, layer),
            is_dirty: false,
            clip_bounds: false,
        }
    }

    pub fn set_from(&mut self, bounds: &AsciiBounds) {
        self.bounds = bounds.clone();
    }
}
