pub mod bounds;
pub mod buffer;
pub mod button;
pub mod character;
pub mod command;
pub mod component;
pub mod position;
pub mod util;


use self::{
    bounds::AsciiBoundsPlugin, button::AsciiButton, character::Character,
    component::AsciiComponentPlugin, position::AsciiPositionPlugin, util::AsciiUtils,
};
use bevy::prelude::*;

//=============================================================================
//             Ascii UI Plugin
//=============================================================================

pub struct AsciiUiPlugin;

impl Plugin for AsciiUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<AsciiUi>()
            .add_plugins(AsciiBoundsPlugin)
            .add_plugins(AsciiPositionPlugin)
            .add_plugins(AsciiUtils)
            .add_plugins(AsciiComponentPlugin::<AsciiButton>::default())
        
            .add_event::<AsciiRerenderUiEvent>()
            .add_systems(PreUpdate, clean_ui)
            .add_systems(PostUpdate, mark_ui_dirty)
        ;
    }
}

//=============================================================================
//             Ascii UiComponent
//=============================================================================

#[derive(Default, Component, Reflect)]
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
//             Rerender Ui Event
//=============================================================================

#[derive(Event, Debug, Clone, Reflect, PartialEq, Eq)]
pub struct AsciiRerenderUiEvent;

fn mark_ui_dirty(
    mut ui: Query<&mut AsciiUi>,
    mut events : EventReader<AsciiRerenderUiEvent>
) {
    if !events.is_empty() {
        for mut ui in ui.iter_mut() {
            ui.is_dirty = true;
        }
    }
}

fn clean_ui( 
    mut ui: Query<&mut AsciiUi>
) {
    for mut ui in ui.iter_mut() {
        ui.is_dirty = false;
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
