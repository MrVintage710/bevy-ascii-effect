use bevy::{prelude::*, utils::HashMap};

use crate::ascii::AsciiCamera;

use super::{position::AsciiPosition, HorizontalAlignment, VerticalAlignment};

//=============================================================================
//             Plugin and Systems
//=============================================================================

pub struct AsciiBoundsPlugin;

impl Plugin for AsciiBoundsPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<AsciiBounds>()
            .register_type::<AsciiGlobalBounds>()
            .add_systems(PostUpdate, (mark_bounds_dirty, update_bounds).chain());
    }
}

pub fn mark_bounds_dirty(
    mut changed_bounds : Query<(Entity, &mut AsciiGlobalBounds, Ref<AsciiBounds>, Option<&Children>)>
) {
    
    let entities = changed_bounds.iter().filter_map(|value| {
        if value.2.is_changed() {
            Some(value.0)
        } else {
            None
        }
    }).collect::<Vec<_>>();
    
    let mut dirty = Vec::new();
    
    for entity in entities {
        let mut children = Vec::new();
        get_children(entity, &mut children, &changed_bounds);
        dirty.append(&mut children);
        dirty.push(entity);
    }
    
    for entity in dirty {
        if let Ok((_, mut global_bounds, _, _)) = changed_bounds.get_mut(entity) {
            global_bounds.is_dirty = true;
        }
    }
}

pub fn get_children(
    current : Entity,
    children_collection : &mut Vec<Entity>,
    query: &Query<(Entity, &mut AsciiGlobalBounds, Ref<AsciiBounds>, Option<&Children>)>
) {
    let Ok((_, _, _, children)) = query.get(current) else { return };
    
    if let Some(children) = children {
        for child in children.iter() {
            children_collection.push(*child);
            get_children(*child, children_collection, query);
        }
    }
}

pub fn update_bounds(
    mut bounded_entities : Query<(Entity, &mut AsciiGlobalBounds, &AsciiBounds, Option<&Parent>)>,
    acsii_cam_query : Query<&AsciiCamera>
) {
    let entities_to_update = bounded_entities.iter_mut().filter_map(|(entity, mut global_bounds, _, _)| {
        if global_bounds.is_dirty {
            global_bounds.is_dirty = false;
            Some(entity)
        } else {
            None
        }
    }).collect::<Vec<_>>();
    // let global_bounds = HashMap::new();
    
    for entity in entities_to_update {
        let new_global_bounds = get_global_bounds(entity, &bounded_entities, &acsii_cam_query);
        if let Ok((_, mut global_bounds, local_bounds, _)) = bounded_entities.get_mut(entity) {
            if let Some(new_global_bounds) = new_global_bounds {
                if new_global_bounds != global_bounds.bounds {
                    global_bounds.bounds = new_global_bounds
                } 
            } else {
                global_bounds.bounds = local_bounds.clone();
            }
        }
    }
}

pub fn get_global_bounds (
    current : Entity,
    global_bounds_query : &Query<(Entity, &mut AsciiGlobalBounds, &AsciiBounds, Option<&Parent>)>,
    acsii_cam_query : &Query<&AsciiCamera>
) -> Option<AsciiBounds> {
    if let Ok(cam) = acsii_cam_query.get(current) {
        let dims = cam.target_res();
        return Some(AsciiBounds::from_dims(dims.x as u32, dims.x as u32));
    }
    
    let Ok((_, global_bounds, local_bounds, parent)) = 
        global_bounds_query.get(current) else { return None; };
    
    if let Some(parent) = parent {
        let bounds = get_global_bounds(**parent, global_bounds_query, acsii_cam_query).unwrap_or(global_bounds.bounds.clone());
        let new_bound = bounds.relative(local_bounds);
        Some(new_bound)
    } else {
        Some(local_bounds.clone())
    }
}

//=============================================================================
//             Ascii Buffer
//=============================================================================

#[derive(Clone, Default, Debug, Reflect, Component, PartialEq, Eq)]
pub struct AsciiBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub layer: u32,
}

impl AsciiBounds {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        AsciiBounds {
            x,
            y,
            width,
            height,
            layer: 0,
        }
    }
    
    pub fn from_dims(width : u32, height : u32) -> Self {
        AsciiBounds {
            x : 0,
            y : 0,
            width,
            height,
            layer : 0,
        }
    }
    
    pub fn with_layer(mut self, layer: u32) -> Self {
        self.layer = layer;
        self
    }

    pub fn is_within(&self, x: i32, y: i32) -> bool {
        x >= self.x && x <= self.x + self.width as i32 && y >= self.y && y <= self.y + self.height as i32
    }
    
    pub fn is_within_local(&self, x : i32, y : i32) -> bool {
        let x = self.x + x;
        let y = self.y + y;
        x >= self.x && x <= self.x + self.width as i32 && y >= self.y && y <= self.y + self.height as i32
    }
    
    pub fn relative(&self, child : &AsciiBounds) -> AsciiBounds {
        AsciiBounds {
            x : self.x + child.x,
            y : self.y + child.y,
            width : child.width,
            height : child.height,
            layer : child.layer + self.layer + 1,
        }
    }
    
    pub fn aligned(&self, width : u32, height : u32, horizontal_alignment : HorizontalAlignment, vertical_alignment : VerticalAlignment) -> AsciiBounds {
        AsciiPosition::create_bounds_aligned(width, height, horizontal_alignment, vertical_alignment, self)
    }
}

//=============================================================================
//             Ascii Global Bounds
//=============================================================================

#[derive(Clone, Default, Debug, Reflect, Component)]
pub struct AsciiGlobalBounds {
    pub bounds : AsciiBounds,
    pub is_dirty : bool,
    pub clip_bounds : bool,
}

impl AsciiGlobalBounds {
    pub fn new(x : i32, y : i32, width : u32, height : u32) -> AsciiGlobalBounds {
        AsciiGlobalBounds {
            bounds : AsciiBounds::new(x, y, width, height),
            is_dirty : false,
            clip_bounds : false,
        }
    }
    
    pub fn set_from(&mut self, bounds : &AsciiBounds) {
        self.bounds = bounds.clone();
    }
}