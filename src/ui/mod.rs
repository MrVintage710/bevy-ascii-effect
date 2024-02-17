pub mod buffer;
pub mod button;
pub mod node;
pub mod command;
pub mod character;


use std::collections::VecDeque;

use bevy::{
    prelude::*, utils::{hashbrown::HashMap}, window::{PrimaryWindow, WindowResized}
};
use crate::ascii::AsciiCamera;
use self::{buffer::AsciiBounds, character::Character, node::AsciiUiNode};

//=============================================================================
//             Ascii UI Plugin
//=============================================================================

pub struct AsciiUiPlugin;

impl Plugin for AsciiUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PostUpdate, (update_is_dirty, update_bounds).chain())
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

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum HorizontalAlignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
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

fn update_bounds(
    ascii_ui: Query<(&AsciiUi, &AsciiCamera, &Children)>,
    mut nodes : Query<(Entity, &mut AsciiUiNode, Option<&Children>)>,
) {
    for (ui, camera, children) in ascii_ui.iter() {
        if ui.is_dirty {
            let target_res = camera.target_res();
            let bounds = AsciiBounds::from_dims(target_res.x as u32, target_res.y as u32);
            let mut map = HashMap::new();
            
           for child in children.iter() {
               calc_bounds(*child, &bounds, &mut map, &nodes);
           }
           
           for (entity, mut node, _) in nodes.iter_mut() {
               if let Some(bounds) = map.remove(&entity) {
                   node.bounds = bounds;
                   node.is_dirty = false;
               }
           }
        }
    }
}

fn calc_bounds(current : Entity, last_bound: &AsciiBounds, map : &mut HashMap<Entity, AsciiBounds>, nodes : &Query<(Entity, &mut AsciiUiNode, Option<&Children>)>) {
    if let Ok((_, node, children)) = nodes.get(current) {
        if let Some(children) = children {
            for child in children.iter() {
                let bound = last_bound.from_layout(node.layout());
                calc_bounds(*child, &bound, map, nodes);
                map.insert(current.clone(), bound);
            }
        }
    }
}

fn update_is_dirty(
    mut ascii_ui: Query<(&mut AsciiUi, &Children)>,
    nodes : Query<(Entity, &mut AsciiUiNode, Option<&Children>)>,
) {
    for (mut ui, children) in ascii_ui.iter_mut() {
        if children.iter().any(|child| is_any_dirty(*child, &nodes)) {
            ui.is_dirty = true;
        }
    }
}

fn is_any_dirty(current : Entity, nodes : &Query<(Entity, &mut AsciiUiNode, Option<&Children>)>) -> bool {
    if let Ok((_, node, children)) = nodes.get(current) {
        if node.is_dirty() {
            return true;
        }
        
        if let Some(children) = children {
            return children.iter().any(|child| is_any_dirty(*child, nodes));
        }
    }
    
    false
}

fn prepare_ui (
    mut ascii_ui: Query<(&mut AsciiUi)>
) {
    for mut ui in ascii_ui.iter_mut() {
        ui.is_dirty = false;
    }
}



// fn update_ui_nodes_recursive(current : Entity, parrent_bounds : &AsciiBounds, nodes : &mut Query<(&mut AsciiUiNode, Option<&Children>)>) -> bool {
//     let mut node_children = Vec::new();
//     let mut is_dirty = false;
    
//     if let Ok((mut node, children)) = nodes.get_mut(current) {
//         let bounds = parrent_bounds.from_layout(node.layout());
//         node.bounds = bounds;
        
//         if let Some(children) = children {
//             node_children = children.iter().collect();
//         }
        
//         is_dirty = node.is_dirty();
//     }
    
//     for child in node_children.iter() {
//         if update_ui_nodes_recursive(**child, parrent_bounds, nodes) {
//             is_dirty = true;
//         }
//     }
    
//     is_dirty
// }

// pub struct TestNode {
//     dims : AsciiBounds,
//     color : Color,
//     agree_btn : AsciiButton,
//     disagree_btn : AsciiButton,
// }

// impl Default for TestNode {
//     fn default() -> Self {
//         TestNode {
//             dims : AsciiBounds::from_dims(40, 20),
//             color : Color::Violet,
//             agree_btn : AsciiButton::new("Agree"),
//             disagree_btn : AsciiButton::new("Disagree"),
//         }
//     }
// }

// impl AsciiUiNode for TestNode {
//     fn render(&mut self, buffer: &AsciiBuffer) {
//         let center = buffer.center(self.dims.width, self.dims.height);
        
//         self.dims = center.clone().bounds;
//         let inner_square = center
//             .square()
//             .border(BorderType::Full)
//             .title("Test Box")
//             .border_color(self.color)
//             .draw();
        
//         if let Some(inner_square) = inner_square {
//             let (top, bottom) = inner_square.top(3);
//             top.padding((0, 0, 1, 0)).text("Are you sure that you want to continue?").horizontal_alignment(HorizontalAlignment::Center).wrap().draw();
//             if let Some(bottom) = bottom {
//                 if let Some(splits) = bottom.vertical_split::<2>() {
//                     self.agree_btn.render(&splits[0].padding((3, 1, 3, 1)));
//                     self.disagree_btn.render(&splits[1].padding((3, 1, 3, 1)));
//                     // splits[0].padding((0, 1, 0, 0)).text("This text should be on the left, and it should wrap to the next line.").wrap().draw();
//                     // splits[1].text("This text should be on the right, and it should wrap to the next line.").wrap().draw();
//                 }
//             }
//         }
//     }

//     fn update(&mut self, context: &mut AsciiUiContext) {
//         // let Some(cursor) = context.cursor_pos() else {return;};
//         // if self.dims.is_within(cursor.0, cursor.1) {
//         //     if self.color != Color::Red {context.mark_dirty()}
//         //     self.color = Color::Red;
//         // } else {
//         //     if self.color != Color::Violet {context.mark_dirty()}
//         //     self.color = Color::Violet;
//         // }
//         self.agree_btn.update(context);
//         self.disagree_btn.update(context);
//     }
// }
