use bevy::{prelude::*, utils::HashSet};

use crate::ascii::AsciiCamera;

use super::{
    bounds::{AsciiBounds, AsciiNode}, util::Value, AsciiMarkDirtyEvent, HorizontalAlignment, Padding, VerticalAlignment
};

//=============================================================================
//            AsciiLayoutPlugin
//=============================================================================

pub struct AsciiPositionPlugin;

impl Plugin for AsciiPositionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AsciiPosition>()
            .add_systems(PostUpdate, (mark_positions_dirty, update_positions).chain());
    }
}

//=============================================================================
//            System for Marking Positions Dirty
//=============================================================================

fn mark_positions_dirty(
    mut changed_bounds: Query<(
        Entity,
        &mut AsciiNode,
        Ref<AsciiPosition>,
        Option<Ref<InheritedVisibility>>,
        Option<&Children>,
    )>,
    mut ui_rerender_event : EventWriter<AsciiMarkDirtyEvent>,
) {
    let entities = changed_bounds
        .iter()
        .filter_map(|value| {
            let v = value.3.map(|value| value.is_changed()).unwrap_or(false);
            if value.2.is_changed() || v  {
                Some(value.0)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();

    let mut dirty = HashSet::new();

    for entity in entities {
        let mut children = Vec::new();
        get_children(entity, &mut children, &changed_bounds);
        children.into_iter().for_each(|child| {
            dirty.insert(child);
        });
        dirty.insert(entity);
    }

    if !dirty.is_empty() {
        ui_rerender_event.send(AsciiMarkDirtyEvent);
    }
    
    for (entity, mut global_bounds, _, vis, _) in changed_bounds.iter_mut() {
        if dirty.contains(&entity) || global_bounds.changed() {
            global_bounds.is_dirty = true;
            global_bounds.clear_changed();
        }
    }
}

fn get_children(
    current: Entity,
    children_collection: &mut Vec<Entity>,
    query: &Query<(
        Entity,
        &mut AsciiNode,
        Ref<AsciiPosition>,
        Option<Ref<InheritedVisibility>>,
        Option<&Children>,
    )>,
) {
    let Ok((_, _, _, _, children)) = query.get(current) else {
        return;
    };

    if let Some(children) = children {
        for child in children.iter() {
            children_collection.push(*child);
            get_children(*child, children_collection, query);
        }
    }
}

//=============================================================================
//            System for Updating Positions
//=============================================================================

fn update_positions(
    mut bounded_entities: Query<(
        Entity,
        &mut AsciiNode,
        Option<&AsciiPosition>,
        Option<&Parent>,
    )>,
    acsii_cam_query: Query<&AsciiCamera>,
) {
    let entities_to_update = bounded_entities
        .iter_mut()
        .filter_map(|(entity, mut global_bounds, _, _)| {
            if global_bounds.is_dirty {
                global_bounds.is_dirty = false;
                Some(entity)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    // let global_bounds = HashMap::new();

    for entity in entities_to_update {
        let new_global_bounds = get_global_bounds(entity, &bounded_entities, &acsii_cam_query);
        if let Ok((_, mut global_bounds, _, _)) = bounded_entities.get_mut(entity) {
            if let Some(new_global_bounds) = new_global_bounds {
                if new_global_bounds != global_bounds.bounds {
                    global_bounds.bounds = new_global_bounds;
                }
            }
        }
    }
}

fn get_global_bounds(
    current: Entity,
    global_bounds_query: &Query<(
        Entity,
        &mut AsciiNode,
        Option<&AsciiPosition>,
        Option<&Parent>,
    )>,
    acsii_cam_query: &Query<&AsciiCamera>,
) -> Option<AsciiBounds> {
    if let Ok(ascii_cam) = acsii_cam_query.get(current) {
        let dims = ascii_cam.target_res();
        return Some(AsciiBounds::from_dims(dims.x as u32, dims.y as u32));
    }

    let Ok((_, global_bounds, local_position, parent)) = global_bounds_query.get(current) else {
        return None
    };

    if let Some(parent) = parent {
        let parent_bounds = get_global_bounds(**parent, global_bounds_query, acsii_cam_query)
            .unwrap_or(global_bounds.bounds.clone());
        if let Some(position) = local_position {
            let new_bound = position.create_bounds(&parent_bounds);
            Some(new_bound)
        } else {
            Some(global_bounds.bounds.clone())
        }
    } else {
        Some(global_bounds.bounds.clone())
    }
}

//=============================================================================
//            AsciiLayouts
//=============================================================================

#[derive(Component, Reflect)]
pub enum AsciiPosition {
    Aligned {
        width: Value,
        height: Value,
        horizontal: HorizontalAlignment,
        vertical: VerticalAlignment,
    },
    Padded {
        padding: Padding,
    },
    VerticalSlice {
        total_silces: u32,
        slice: u32,
    },
    HorizontalSlice {
        total_silces: u32,
        slice: u32,
    },
    Relative {
        x : i32,
        y : i32,
        width: Value,
        height: Value,
        layer : u32,
    },
    Absolute {
        bounds: AsciiBounds,
    },
}

impl AsciiPosition {
    pub fn relative(x: i32, y: i32, width: impl Into<Value>, height: impl Into<Value>, layer : u32) -> Self {
        AsciiPosition::Relative {
            x,
            y,
            width: width.into(),
            height: height.into(),
            layer,
        }
    }
    
    pub fn top(size : impl Into<Value>) -> Self {
        AsciiPosition::Aligned { 
            width: 1.0.into(),
            height: size.into(), 
            horizontal: HorizontalAlignment::Center, 
            vertical: VerticalAlignment::Top 
        }
    }
    
    pub fn bottom(size : impl Into<Value>) -> Self {
        AsciiPosition::Aligned { 
            width: 1.0.into(),
            height: size.into(), 
            horizontal: HorizontalAlignment::Center, 
            vertical: VerticalAlignment::Bottom 
        }
    }
    
    pub fn left(size : impl Into<Value>) -> Self {
        AsciiPosition::Aligned { 
            width: size.into(),
            height: 1.0.into(), 
            horizontal: HorizontalAlignment::Left, 
            vertical: VerticalAlignment::Center 
        }
    }
    
    pub fn right(size : impl Into<Value>) -> Self {
        AsciiPosition::Aligned { 
            width: size.into(),
            height: 1.0.into(), 
            horizontal: HorizontalAlignment::Right, 
            vertical: VerticalAlignment::Center 
        }
    }
    
    pub fn centered(width: impl Into<Value>, height: impl Into<Value>) -> Self {
        AsciiPosition::Aligned { 
            width: width.into(),
            height: height.into(), 
            horizontal: HorizontalAlignment::Center, 
            vertical: VerticalAlignment::Center 
        }
    }
    
    pub fn fill() -> Self {
        AsciiPosition::Aligned { 
            width: 1.0.into(),
            height: 1.0.into(), 
            horizontal: HorizontalAlignment::Center, 
            vertical: VerticalAlignment::Center 
        }
    }

    pub fn aligned(
        width: impl Into<Value>,
        height: impl Into<Value>,
        horizontal: HorizontalAlignment,
        vertical: VerticalAlignment,
    ) -> Self {
        AsciiPosition::Aligned {
            width : width.into(),
            height : height.into(),
            horizontal,
            vertical,
        }
    }

    pub fn format_bounds(&self, parent_bounds: &AsciiBounds, child_bounds: &mut AsciiBounds) {
        match self {
            AsciiPosition::Aligned {
                width,
                height,
                horizontal,
                vertical,
            } => Self::format_bounds_aligned(
                *width,
                *height,
                *horizontal,
                *vertical,
                parent_bounds,
                child_bounds,
            ),
            AsciiPosition::Padded { padding } => {
                Self::format_bounds_padded(*padding, parent_bounds, child_bounds)
            }
            AsciiPosition::VerticalSlice {
                total_silces,
                slice,
            } => {
                // Self::format_bounds_vertical_slice(*total_silces, *slice, parent_bounds)
                todo!()
            }
            AsciiPosition::HorizontalSlice {
                total_silces,
                slice,
            } => {
                // Self::create_bounds_horizontal_slice(*total_silces, *slice, parent_bounds)
                todo!()
            }
            AsciiPosition::Relative { x, y, width, height, layer } => {
                Self::format_bounds_relative(*x, *y, *width, *height, *layer, parent_bounds, child_bounds)
            }
            AsciiPosition::Absolute { bounds } => todo!(),
        }
    }

    pub fn create_bounds(&self, parent_bounds: &AsciiBounds) -> AsciiBounds {
        match self {
            AsciiPosition::Aligned {
                width,
                height,
                horizontal,
                vertical,
            } => {
                Self::create_bounds_aligned(*width, *height, *horizontal, *vertical, parent_bounds)
            }
            AsciiPosition::Padded { padding } => {
                todo!()
            }
            AsciiPosition::VerticalSlice {
                total_silces,
                slice,
            } => {
                todo!()
            }
            AsciiPosition::HorizontalSlice {
                total_silces,
                slice,
            } => {
                todo!()
            }
            AsciiPosition::Relative { x, y, width, height, layer  } => {
                Self::create_bounds_relative(*x, *y, *width, *height, *layer, parent_bounds)
            }
            AsciiPosition::Absolute { bounds } => {
                todo!()
            }
        }
    }

    pub fn create_bounds_aligned(
        width: impl Into<Value>,
        height: impl Into<Value>,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
        parent_bounds: &AsciiBounds,
    ) -> AsciiBounds {
        let mut child = AsciiBounds::default();
        Self::format_bounds_aligned(
            width,
            height,
            horizontal_alignment,
            vertical_alignment,
            parent_bounds,
            &mut child,
        );
        child
    }

    pub fn format_bounds_aligned(
        width: impl Into<Value>,
        height: impl Into<Value>,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
        parent_bounds: &AsciiBounds,
        child_bounds: &mut AsciiBounds,
    ) {
        let width = width.into().pixel_u32(parent_bounds.width);
        let height = height.into().pixel_u32(parent_bounds.height);
        
        let x = match horizontal_alignment {
            HorizontalAlignment::Left => 0,
            HorizontalAlignment::Center => ((parent_bounds.width as f32 / 2.0)
                - (width as f32 / 2.0))
                .floor()
                .max(0.0) as i32,
            HorizontalAlignment::Right => parent_bounds.width.saturating_sub(width) as i32,
        };
        let y = match vertical_alignment {
            VerticalAlignment::Top => 0,
            VerticalAlignment::Center => ((parent_bounds.height as f32 / 2.0)
                - (height as f32 / 2.0))
                .floor()
                .max(0.0) as i32,
            VerticalAlignment::Bottom => parent_bounds.height.saturating_sub(height) as i32,
        };
        child_bounds.x = parent_bounds.x + x;
        child_bounds.y = parent_bounds.y + y;
        child_bounds.width = width.min(parent_bounds.width);
        child_bounds.height = height.min(parent_bounds.height);
        child_bounds.layer = parent_bounds.layer + 1;
    }

    fn format_bounds_padded(
        padding: impl Into<Padding>,
        parent_bounds: &AsciiBounds,
        child_bounds: &mut AsciiBounds,
    ) {
        let padding = padding.into();
        let horizontal_difference =
            parent_bounds.width as i32 - padding.left as i32 - padding.right as i32;
        let vertical_difference =
            parent_bounds.height as i32 - padding.top as i32 - padding.bottom as i32;
        if horizontal_difference <= 0 || vertical_difference <= 0 {
            return;
        }
        child_bounds.x = padding.left as i32;
        child_bounds.y = padding.top as i32;
        child_bounds.width = parent_bounds.width - (padding.left + padding.right);
        child_bounds.height = parent_bounds.height - (padding.top + padding.bottom);
    }

    fn format_bounds_vertical_slice(
        total_silces: u32,
        slice: u32,
        parent_bounds: &AsciiBounds,
        child_bounds: &mut AsciiBounds,
    ) {
        todo!()
    }

    fn format_bounds_horizontal_slice(
        total_silces: u32,
        slice: u32,
        parent_bounds: &AsciiBounds,
        child_bounds: &mut AsciiBounds,
    ) {
        todo!()
    }

    fn format_bounds_relative(
        x: i32, y : i32, width : impl Into<Value>, height : impl Into<Value>, layer : u32,
        parent_bounds: &AsciiBounds,
        child_bounds: &mut AsciiBounds,
    ) {
        let width = width.into().pixel_u32(parent_bounds.width);
        let height = height.into().pixel_u32(parent_bounds.height);
        
        child_bounds.x = x + parent_bounds.x;
        child_bounds.y = y + parent_bounds.y;
        child_bounds.width = width;
        child_bounds.height = height;
        child_bounds.layer = layer + parent_bounds.layer + 1;
    }

    fn create_bounds_relative(x: i32, y : i32, width : impl Into<Value>, height : impl Into<Value>, layer : u32, parent_bounds: &AsciiBounds) -> AsciiBounds {
        let mut child = AsciiBounds::default();
        Self::format_bounds_relative(x, y, width, height, layer, parent_bounds, &mut child);
        child
    }
}
