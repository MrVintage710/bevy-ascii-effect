use std::marker::PhantomData;

use bevy::{ecs::query::WorldQuery, prelude::*, render::{extract_component, view::RenderLayers, Extract, RenderApp}};

use crate::{ascii::AsciiCamera, render::{ascii::OverlayBuffer, extract_camera}};

use super::{bounds::{AsciiBounds, AsciiGlobalBounds}, buffer::AsciiBuffer, node::AsciiNode, AsciiUi};

//=============================================================================
//             Component Plugin
//=============================================================================

pub struct AsciiComponentPlugin<AC : AsciiComponent>(PhantomData<AC>);

impl <AC : AsciiComponent> Default for AsciiComponentPlugin<AC> {
    fn default() -> Self {
        AsciiComponentPlugin(PhantomData)
    }
}

impl<AC : AsciiComponent> Plugin for AsciiComponentPlugin<AC> {
    fn build(&self, app: &mut App) {
        AC::set_up(app);
        app.add_systems(Update, update_components::<AC>);
        
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        
        render_app.add_systems(ExtractSchedule, extract_ascii_ui::<AC>.after(apply_deferred));
    }
}

//=============================================================================
//             Componet Systems
//=============================================================================

fn update_components<C : AsciiComponent>(
    mut nodes : Query<(&mut C, &AsciiGlobalBounds)>,
    mut query : Query<C::UpdateQuery>
) {
    for mut item in query.iter_mut() {
        for (mut component, global_bounds) in nodes.iter_mut() {
            component.update(&mut item, &global_bounds.bounds);
        }
    }
}

pub fn extract_ascii_ui<C : AsciiComponent>(
    mut commands: Commands,
    mut ascii_cameras: Query<(Entity, &AsciiCamera, &mut OverlayBuffer, Option<&RenderLayers>)>,
    ui_elements: Extract<Query<(&AsciiNode, &AsciiGlobalBounds, &C, Option<&RenderLayers>)>>,
) {
    // println!("Extracting Ascii UI | {}, {}", ascii_cameras.iter().count(), ui_elements.iter().count());
    for (cam_entity, node, mut buffer, camera_render_layers) in ascii_cameras.iter_mut() {
        for (node, global_bounds, component, component_render_layer) in ui_elements.iter() {
            match (component_render_layer, camera_render_layers) {
                (Some(_), None) |
                (None, Some(_)) => {
                    continue;
                },
                (Some(component_render_layer), Some(camera_render_layers)) => {
                    if !component_render_layer.intersects(camera_render_layers) {
                        continue;
                    }
                }
                (None, None) => {}
            }
            
            let surface = &buffer.0;
            let mut buffer = AsciiBuffer::new(surface, &global_bounds.bounds);
            
            // println!("Rendering component");
            component.render(&mut buffer);
        }
    }
}

//=============================================================================
//             AsciiComponent Trait
//=============================================================================

pub trait AsciiComponent : Component {
    type UpdateQuery : WorldQuery;
    
    fn render(&self, buffer : &mut AsciiBuffer) {}
    
    fn update(&mut self, query : &mut <Self::UpdateQuery as WorldQuery>::Item<'_>, bounds : &AsciiBounds) {}
    
    fn set_up(app : &mut App) {}
}