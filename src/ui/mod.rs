pub mod buffer;
pub mod button;
pub mod node;
pub mod command;
pub mod character;
pub mod component;
pub mod bounds;


use std::collections::VecDeque;

use bevy::{
    prelude::*, utils::{hashbrown::HashMap}, window::{PrimaryWindow, WindowResized}
};
use crate::ascii::AsciiCamera;
use self::{bounds::AsciiBoundsPlugin, buffer::AsciiSurface, button::AsciiButton, character::Character, component::AsciiComponentPlugin, node::AsciiNode};

//=============================================================================
//             Ascii UI Plugin
//=============================================================================

pub struct AsciiUiPlugin;

impl Plugin for AsciiUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(AsciiBoundsPlugin)
            .add_plugins(AsciiComponentPlugin::<AsciiButton>::default())
        ;
        
        app
            .register_type::<AsciiNode>()
            // .add_systems(PostUpdate, (update_is_dirty, update_bounds).chain())
            .add_systems(PreUpdate, prepare_ui)
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

//=============================================================================
//             Ascii Ui Node
//=============================================================================

// fn update_bounds(
//     ascii_ui: Query<(&AsciiUi, &AsciiCamera, &Children)>,
//     mut nodes : Query<(Entity, &mut AsciiNode, Option<&Children>)>,
// ) {
//     for (ui, camera, children) in ascii_ui.iter() {
//         if ui.is_dirty {
//             let target_res = camera.target_res();
//             let bounds = AsciiBounds::from_dims(target_res.x as u32, target_res.y as u32);
//             let mut map = HashMap::new();
            
//             let mut iteration = 0;
//            for child in children.iter() {
//                calc_bounds(*child, &bounds, &mut map, &nodes, &mut iteration);
//            }
           
           
//            for (entity, mut node, _) in nodes.iter_mut() {
//                if let Some((bounds, order)) = map.remove(&entity) {
//                    node.bounds = bounds;
//                    node.render_order = order;
//                    node.is_dirty = false;
//                }
//            }
//         }
//     }
// }

// fn calc_bounds(current : Entity, last_bound: &AsciiBounds, map : &mut HashMap<Entity, (AsciiBounds, u32)>, nodes : &Query<(Entity, &mut AsciiUiNode, Option<&Children>)>, iteration : &mut u32) {
//     if let Ok((_, node, children)) = nodes.get(current) {
//         let bound = last_bound.from_layout(node.layout());
//         if let Some(children) = children {
//             for child in children.iter() {
//                 calc_bounds(*child, &bound, map, nodes, iteration);
//             }
//         }
//         map.insert(current.clone(), (bound, *iteration));
//         *iteration += 1;
//     }
// }

// fn update_is_dirty(
//     mut ascii_ui: Query<(&mut AsciiUi, &Children)>,
//     nodes : Query<(Entity, &mut AsciiUiNode, Option<&Children>)>,
// ) {
//     for (mut ui, children) in ascii_ui.iter_mut() {
//         if children.iter().any(|child| is_any_dirty(*child, &nodes)) {
//             ui.is_dirty = true;
//         }
//     }
// }

// fn is_any_dirty(current : Entity, nodes : &Query<(Entity, &mut AsciiNode, Option<&Children>)>) -> bool {
//     if let Ok((_, node, children)) = nodes.get(current) {
//         if node.is_dirty() {
//             return true;
//         }
        
//         if let Some(children) = children {
//             return children.iter().any(|child| is_any_dirty(*child, nodes));
//         }
//     }
    
//     false
// }

fn prepare_ui (
    mut ascii_ui: Query<(&mut AsciiUi)>,
    mut resized_event : EventReader<WindowResized>
) {
    for mut ui in ascii_ui.iter_mut() {
        if !resized_event.is_empty() {
            ui.is_dirty = true;
        } else {
            ui.is_dirty = false;
        }
        
    }
}
