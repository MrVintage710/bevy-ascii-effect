
use bevy::{ecs::query::WorldQuery, prelude::*};

use super::{bounds::AsciiBounds, buffer::AsciiBuffer};

//=============================================================================
//             AsciiNodePlugin
//=============================================================================

pub struct AsciiNodePlugin;

impl Plugin for AsciiNodePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MarkNodeDirtyEvent>()
        ;
    }
}

//=============================================================================
//             AsciiNode
//=============================================================================

#[derive(Default, Component, Reflect)]
pub struct AsciiNode {
    pub clip_bounds : bool,
    pub is_dirty : bool,
    pub order : u32,
}

//=============================================================================
//             Mark Node Dirty Event
//=============================================================================

#[derive(Event)]
pub struct MarkNodeDirtyEvent(pub Entity);



