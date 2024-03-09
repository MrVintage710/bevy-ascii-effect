pub mod bounds;
pub mod buffer;
pub mod component;
pub mod character;
pub mod command;
pub mod position;
pub mod util;


use self::{
    bounds::AsciiBoundsPlugin, character::Character,
    component::AsciiComponentPlugin, position::AsciiPositionPlugin, util::AsciiUtils,
};

use self::component::button::AsciiButton;

use bevy::prelude::*;
use bevy::window::WindowResized;

//=============================================================================
//             Ascii UI Plugin
//=============================================================================

pub struct AsciiUiPlugin;

impl Plugin for AsciiUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(AsciiBoundsPlugin)
            .add_plugins(AsciiPositionPlugin)
            .add_plugins(AsciiUtils)
            .add_plugins(AsciiComponentPlugin::<AsciiButton>::default())
        
            .add_event::<AsciiMarkDirtyEvent>()
            .add_systems(PreUpdate, clean_ui)
            .add_systems(PostUpdate, mark_ui_dirty)
            
            .register_type::<AsciiUi>()
            .register_type::<AsciiButton>()
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
pub struct AsciiMarkDirtyEvent;

fn mark_ui_dirty(
    mut ui: Query<&mut AsciiUi>,
    mut events : EventReader<AsciiMarkDirtyEvent>,
    window_events : EventReader<WindowResized>
) {
    if !events.is_empty() || !window_events.is_empty(){
        for mut ui in ui.iter_mut() {
            ui.is_dirty = true;
        }
    }
    
    events.clear();
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
        if x == 0 && y == 0 {
            self.top_left()
        } else if x == width - 1 && y == 0 {
            self.top_right()
        } else if x == 0 && y == height - 1 {
            self.bottom_left()
        } else if x == width - 1 && y == height - 1 {
            self.bottom_right()
        } else if x == 0 {
            self.left()
        } else if x == width - 1 {
            self.right()
        } else if y == 0 {
            self.top()
        } else if y == height - 1 {
            self.bottom()
        } else {
            Character::Nil
        }
    }
    
    pub fn top(&self) -> Character {
        match self {
            BorderType::Full => Character::BorderN,
            BorderType::Half => todo!(),
            BorderType::Dashed => todo!(),
            BorderType::None => Character::Nil,
        }
    }
    
    pub fn bottom(&self) -> Character {
        match self {
            BorderType::Full => Character::BorderS,
            BorderType::Half => todo!(),
            BorderType::Dashed => todo!(),
            BorderType::None => Character::Nil,
        }
    }
    
    pub fn left(&self) -> Character {
        match self {
            BorderType::Full => Character::BorderW,
            BorderType::Half => todo!(),
            BorderType::Dashed => todo!(),
            BorderType::None => Character::Nil,
        }
    }
    
    pub fn right(&self) -> Character {
        match self {
            BorderType::Full => Character::BorderE,
            BorderType::Half => todo!(),
            BorderType::Dashed => todo!(),
            BorderType::None => Character::Nil,
        }
    }
    
    pub fn top_left(&self) -> Character {
        match self {
            BorderType::Full => Character::LBorderNW,
            BorderType::Half => todo!(),
            BorderType::Dashed => todo!(),
            BorderType::None => Character::Nil,
        }
    }
    
    pub fn top_right(&self) -> Character {
        match self {
            BorderType::Full => Character::LBorderNE,
            BorderType::Half => todo!(),
            BorderType::Dashed => todo!(),
            BorderType::None => Character::Nil,
        }
    }
    
    pub fn bottom_left(&self) -> Character {
        match self {
            BorderType::Full => Character::LBorderSW,
            BorderType::Half => todo!(),
            BorderType::Dashed => todo!(),
            BorderType::None => Character::Nil,
        }
    }
    
    pub fn bottom_right(&self) -> Character {
        match self {
            BorderType::Full => Character::LBorderSE,
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
