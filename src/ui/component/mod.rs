pub mod button;

use std::marker::PhantomData;

use bevy::{
    ecs::
        system::{StaticSystemParam, SystemParam}
    ,
    prelude::*,
    render::{view::{visibility, RenderLayers}, Extract, RenderApp},
};

use crate::{
    ascii::AsciiCamera,
    render::ascii::OverlayBuffer,
};

use self::button::AsciiButton;

use super::{
    bounds::{AsciiBounds, AsciiNode},
    buffer::AsciiBuffer, AsciiMarkDirtyEvent,
};

//=============================================================================
//             Components Plugin
//=============================================================================

pub struct AsciiDefaultComponentsPlugin;

impl Plugin for AsciiDefaultComponentsPlugin {
    fn build(&self, app: &mut App) {
        app
            
            .add_plugins(AsciiComponentPlugin::<AsciiButton>::default())
            .register_type::<AsciiButton>()
        ;
    }
}

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
    mut nodes: Query<(Entity, &mut C, &AsciiNode, Option<&InheritedVisibility>)>,
    mut query: StaticSystemParam<C::UpdateQuery<'_, '_>>,
) {
    for (entity, mut component, global_bounds, visibility) in nodes.iter_mut() {
        let is_visible = visibility.map(|v| v.get()).unwrap_or(true);
        if !is_visible {
            continue;
        }
        component.update(&mut (*query), &global_bounds.bounds, entity);
    }
}

pub fn extract_ascii_ui<C: AsciiComponent>(
    ascii_cameras: Query<(&OverlayBuffer, Option<&RenderLayers>), With<AsciiCamera>>,
    ui_elements: Extract<Query<(&AsciiNode, &C, Option<&RenderLayers>, Option<&InheritedVisibility>)>>,
) {    
    for (buffer, camera_render_layers) in ascii_cameras.iter() {
        for (global_bounds, component, component_render_layer, visibility) in ui_elements.iter() {
            if let Some(visibility) = visibility {
                if !visibility.get() {
                    continue;
                }
            }
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
            let mut buffer = AsciiBuffer::new(surface, &global_bounds.bounds, None);
            
            component.render(&mut buffer);
        }
    }
}

//=============================================================================
//             AsciiComponent Trait
//=============================================================================

pub trait AsciiComponent: Component {
    type UpdateQuery<'w, 's> : SystemParam;

    #[allow(unused_variables)]
    fn render(&self, buffer: &mut AsciiBuffer) {}

    #[allow(unused_variables)]
    fn update(
        &mut self,
        query: &mut <Self::UpdateQuery<'_, '_> as SystemParam>::Item<'_, '_>,
        bounds: &AsciiBounds,
        entity: Entity
    ) {}

    #[allow(unused_variables)]
    fn set_up(app: &mut App) {}
}