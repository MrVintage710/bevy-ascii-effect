use bevy::{prelude::*, utils::HashSet};

use crate::ascii::AsciiCamera;

use super::{
    bounds::{AsciiBounds, AsciiGlobalBounds}, AsciiRerenderUiEvent, HorizontalAlignment, Padding, VerticalAlignment
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
        &mut AsciiGlobalBounds,
        Ref<AsciiPosition>,
        Option<&Children>,
    )>,
    mut ui_rerender_event : EventWriter<AsciiRerenderUiEvent>,
) {
    let entities = changed_bounds
        .iter()
        .filter_map(|value| {
            if value.2.is_changed() {
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
        ui_rerender_event.send(AsciiRerenderUiEvent);
    }
    
    for (entity, mut global_bounds, _, _) in changed_bounds.iter_mut() {
        if dirty.contains(&entity) || global_bounds.is_changed() {
            global_bounds.is_dirty = true;
        }
    }
}

fn get_children(
    current: Entity,
    children_collection: &mut Vec<Entity>,
    query: &Query<(
        Entity,
        &mut AsciiGlobalBounds,
        Ref<AsciiPosition>,
        Option<&Children>,
    )>,
) {
    let Ok((_, global_bounds, _, children)) = query.get(current) else {
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
        &mut AsciiGlobalBounds,
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
        if let Ok((_, mut global_bounds, local_bounds, _)) = bounded_entities.get_mut(entity) {
            if let Some(new_global_bounds) = new_global_bounds {
                if new_global_bounds != global_bounds.bounds {
                    global_bounds.bounds = new_global_bounds
                }
            }
            // else {
            //     global_bounds.bounds = local_bounds.clone();
            // }
        }
    }
}

fn get_global_bounds(
    current: Entity,
    global_bounds_query: &Query<(
        Entity,
        &mut AsciiGlobalBounds,
        Option<&AsciiPosition>,
        Option<&Parent>,
    )>,
    acsii_cam_query: &Query<&AsciiCamera>,
) -> Option<AsciiBounds> {
    if let Ok(cam) = acsii_cam_query.get(current) {
        let dims = cam.target_res();
        return Some(AsciiBounds::from_dims(dims.x as u32, dims.x as u32));
    }

    let Ok((_, global_bounds, local_position, parent)) = global_bounds_query.get(current) else {
        return None;
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
        width: u32,
        height: u32,
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
        bounds: AsciiBounds,
    },
    Absolute {
        bounds: AsciiBounds,
    },
}

impl AsciiPosition {
    pub fn relavtive(x: i32, y: i32, width: u32, height: u32) -> Self {
        AsciiPosition::Relative {
            bounds: AsciiBounds::new(x, y, width, height),
        }
    }

    pub fn align(
        width: u32,
        height: u32,
        horizontal: HorizontalAlignment,
        vertical: VerticalAlignment,
    ) -> Self {
        AsciiPosition::Aligned {
            width,
            height,
            horizontal,
            vertical,
        }
    }

    pub fn format_bounds(&self, parent_bounds: &mut AsciiBounds, child_bounds: &mut AsciiBounds) {
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
            }
            AsciiPosition::HorizontalSlice {
                total_silces,
                slice,
            } => {
                // Self::create_bounds_horizontal_slice(*total_silces, *slice, parent_bounds)
            }
            AsciiPosition::Relative { bounds } => {
                Self::format_bounds_relative(bounds, parent_bounds, child_bounds)
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
            AsciiPosition::Relative { bounds } => {
                Self::create_bounds_relative(bounds, parent_bounds)
            }
            AsciiPosition::Absolute { bounds } => {
                todo!()
            }
        }
    }

    pub fn create_bounds_aligned(
        width: u32,
        height: u32,
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
        width: u32,
        height: u32,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
        parent_bounds: &AsciiBounds,
        child_bounds: &mut AsciiBounds,
    ) {
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
        bounds: &AsciiBounds,
        parent_bounds: &AsciiBounds,
        child_bounds: &mut AsciiBounds,
    ) {
        child_bounds.x = bounds.x + parent_bounds.x;
        child_bounds.y = bounds.y + parent_bounds.y;
        child_bounds.width = bounds.width;
        child_bounds.height = bounds.height;
        child_bounds.layer = bounds.layer + parent_bounds.layer + 1;
    }

    fn create_bounds_relative(bounds: &AsciiBounds, parent_bounds: &AsciiBounds) -> AsciiBounds {
        let mut child = AsciiBounds::default();
        Self::format_bounds_relative(bounds, parent_bounds, &mut child);
        child
    }
}
