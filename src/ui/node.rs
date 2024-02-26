
use bevy::{ecs::query::WorldQuery, prelude::*};

use super::{bounds::AsciiBounds, buffer::AsciiBuffer};

//=============================================================================
//             AsciiNode
//=============================================================================

#[derive(Default, Component, Reflect)]
pub struct AsciiNode {
    pub clip_bounds : bool,
    pub is_dirty : bool,
    pub order : u32
}

//=============================================================================
//             AsciiUiComponent
//=============================================================================





