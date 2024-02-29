use std::{io::IsTerminal, marker::PhantomData};

use bevy::{
    ecs::{
        query::WorldQuery,
        system::{StaticSystemParam, SystemParam},
    },
    prelude::*,
    render::{extract_component, view::RenderLayers, Extract, RenderApp},
};

use crate::{
    ascii::AsciiCamera,
    render::{ascii::OverlayBuffer, extract_camera},
};

use super::{
    bounds::{AsciiBounds, AsciiGlobalBounds},
    buffer::AsciiBuffer,
    node::AsciiNode,
    AsciiUi,
};

//=============================================================================
//             Component Plugin
//=============================================================================

pub struct AsciiComponentPlugin<AC: AsciiComponent>(PhantomData<AC>);

impl<AC: AsciiComponent> Default for AsciiComponentPlugin<AC> {
    fn default() -> Self {
        AsciiComponentPlugin(PhantomData)
    }
}

impl<AC: AsciiComponent> Plugin for AsciiComponentPlugin<AC> {
    fn build(&self, app: &mut App) {
        AC::set_up(app);
        app.add_systems(Update, update_components::<AC>);

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(
            ExtractSchedule,
            extract_ascii_ui::<AC>.after(apply_deferred),
        );
    }
}

//=============================================================================
//             Componet Systems
//=============================================================================

fn update_components<C: AsciiComponent>(
    mut nodes: Query<(&mut C, &AsciiGlobalBounds)>,
    mut query: StaticSystemParam<C::UpdateQuery>,
) {
    // let query = *query;
    for (mut component, global_bounds) in nodes.iter_mut() {
        component.update(&mut (*query), &global_bounds.bounds);
    }
}

pub fn extract_ascii_ui<C: AsciiComponent>(
    ascii_cameras: Query<(&OverlayBuffer, Option<&RenderLayers>), With<AsciiCamera>>,
    ui_elements: Extract<Query<(&AsciiGlobalBounds, &C, Option<&RenderLayers>)>>,
) {
    for (buffer, camera_render_layers) in ascii_cameras.iter() {
        for (global_bounds, component, component_render_layer) in ui_elements.iter() {
            match (component_render_layer, camera_render_layers) {
                (Some(_), None) | (None, Some(_)) => {
                    continue;
                }
                (Some(component_render_layer), Some(camera_render_layers)) => {
                    if !component_render_layer.intersects(camera_render_layers) {
                        continue;
                    }
                }
                (None, None) => {}
            }

            let surface = &buffer.0;
            let mut buffer = AsciiBuffer::new(surface, &global_bounds.bounds);

            component.render(&mut buffer);
        }
    }
}

//=============================================================================
//             AsciiComponent Trait
//=============================================================================

pub trait AsciiComponent: Component {
    type UpdateQuery: SystemParam;

    fn render(&self, buffer: &mut AsciiBuffer) {}

    fn update(
        &mut self,
        query: &mut <Self::UpdateQuery as SystemParam>::Item<'_, '_>,
        bounds: &AsciiBounds,
    ) {
    }

    fn set_up(app: &mut App) {}
}
