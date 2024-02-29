pub mod bounds;
pub mod buffer;
pub mod button;
pub mod character;
pub mod command;
pub mod component;
pub mod node;
pub mod position;
pub mod util;

use crate::TestEvent;

use self::{
    bounds::AsciiBoundsPlugin, button::AsciiButton, character::Character,
    component::AsciiComponentPlugin, node::AsciiNode, position::AsciiPositionPlugin, util::AsciiUtils,
};
use bevy::prelude::*;

//=============================================================================
//             Ascii UI Plugin
//=============================================================================

pub struct AsciiUiPlugin;

impl Plugin for AsciiUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AsciiBoundsPlugin)
            .add_plugins(AsciiPositionPlugin)
            .add_plugins(AsciiUtils)
            .add_plugins(AsciiComponentPlugin::<AsciiButton>::default());

        app
            .register_type::<AsciiNode>()
            // .add_systems(PostUpdate, (update_is_dirty, update_bounds).chain())
            // .add_systems(PreUpdate, prepare_ui)
        ;
    }
}

//=============================================================================
//             Ascii UiComponent
//=============================================================================

#[derive(Default, Component)]
pub struct AsciiUi {
    // nodes: Vec<Arc<Mutex<Box<dyn AsciiUiNode + Send + Sync>>>>,
    is_dirty: bool,
}

impl AsciiUi {
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }
}

//=============================================================================
//             Styling Constants
//=============================================================================

pub enum BorderType {
    Full,
    Half,
    Dashed,
    None,
}

impl BorderType {
    fn get_character(&self, x: u32, y: u32, width: u32, height: u32) -> Character {
        match self {
            BorderType::Full => {
                if x == 0 && y == 0 {
                    Character::LBorderNW
                } else if x == width - 1 && y == 0 {
                    Character::LBorderNE
                } else if x == 0 && y == height - 1 {
                    Character::LBorderSW
                } else if x == width - 1 && y == height - 1 {
                    Character::LBorderSE
                } else if x == 0 {
                    Character::BorderW
                } else if x == width - 1 {
                    Character::BorderE
                } else if y == 0 {
                    Character::BorderN
                } else if y == height - 1 {
                    Character::BorderS
                } else {
                    Character::Nil
                }
            }
            BorderType::Half => todo!(),
            BorderType::Dashed => todo!(),
            BorderType::None => Character::Nil,
        }
    }
}

#[derive(Default)]
pub enum TextOverflow {
    #[default]
    Hidden,
    Elipses,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Reflect)]
pub enum HorizontalAlignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Reflect)]
pub enum VerticalAlignment {
    #[default]
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub struct Padding {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Default for Padding {
    fn default() -> Self {
        Padding {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }
}

impl From<u32> for Padding {
    fn from(padding: u32) -> Self {
        Padding {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }
}

impl From<(u32, u32)> for Padding {
    fn from((top, right): (u32, u32)) -> Self {
        Padding {
            top,
            right,
            bottom: top,
            left: right,
        }
    }
}

impl From<(u32, u32, u32, u32)> for Padding {
    fn from((top, right, bottom, left): (u32, u32, u32, u32)) -> Self {
        Padding {
            top,
            right,
            bottom,
            left,
        }
    }
}
