use bevy::{
    prelude::*,
    render::renderer::{RenderDevice, RenderQueue},
};

pub(crate) struct DitherShaderPipeline {}

// impl FromWorld for DitherShaderPipeline {
//     fn from_world(world: &mut World) -> Self {
//         let queue = world.resource::<RenderQueue>();
//         let render_device = world.resource::<RenderDevice>();
//     }
// }
