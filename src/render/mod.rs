pub mod ascii;
mod dither;
mod pixel;

use bevy::{
    app::Plugin,
    core_pipeline::{core_3d, prepass::ViewPrepassTextures},
    prelude::*,
    render::{
        self,
        render_graph::{RenderGraphApp, ViewNode, ViewNodeRunner},
        render_resource::{
            BindGroupEntries, Extent3d, ImageCopyTexture, ImageDataLayout, Operations, Origin3d,
            PipelineCache, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
            TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
            TextureView, TextureViewDescriptor,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ExtractedWindows, PostProcessWrite, RenderLayers, ViewTarget},
        Extract, Render, RenderApp, RenderSet,
    },
};
use bevy_inspector_egui::{quick::ResourceInspectorPlugin, InspectorOptions};

use crate::{
    ascii::{AsciiCamera, AsciiShaderSettingsBuffer},
    ui::{bounds::AsciiGlobalBounds, buffer::{AsciiBuffer, AsciiSurface}, node::AsciiNode, AsciiUi},
};

use self::{
    ascii::{AsciiShaderPipeline, OverlayBuffer},
    pixel::PixelShaderPipeline,
};

//=============================================================================
//             Ascii Shader Node
//=============================================================================

pub(crate) struct AsciiRendererPlugin;

impl Plugin for AsciiRendererPlugin {
    fn build(&self, app: &mut App) {
        // We need to get the render app from the main app
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(
                Render,
                prepare_shader_textures.in_set(RenderSet::PrepareResources),
            )
            .add_systems(ExtractSchedule, (extract_camera, apply_deferred))
            .add_render_graph_node::<ViewNodeRunner<AsciiShaderNode>>(
                core_3d::graph::NAME,
                AsciiShaderNode::NAME,
            )
            .add_render_graph_edges(
                core_3d::graph::NAME,
                &[
                    core_3d::graph::node::TONEMAPPING,
                    AsciiShaderNode::NAME,
                    core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
                ],
            );
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            // Initialize the pipeline
            .init_resource::<AsciiShaderPipeline>()
            .init_resource::<PixelShaderPipeline>();
    }
}

//=============================================================================
//             Ascii Shader Node
//=============================================================================

#[derive(Default)]
pub struct AsciiShaderNode;
impl AsciiShaderNode {
    const NAME: &'static str = "AsciiShaderNode";
}

impl ViewNode for AsciiShaderNode {
    type ViewQuery = (Entity, &'static ViewTarget, &'static AsciiCamera);

    fn run(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        view_query: bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let (entity, view_target, ascii_camera) = view_query;

        // Get the pipeline resource that contains the global data we need
        // to create the render pipeline
        let ascii_pipeline_resource = world.resource::<AsciiShaderPipeline>();
        let pixel_pipeline_resource = world.resource::<PixelShaderPipeline>();

        // The pipeline cache is a cache of all previously created pipelines.
        // It is required to avoid creating a new pipeline each frame,
        // which is expensive due to shader compilation.
        let pipeline_cache = world.resource::<PipelineCache>();

        // Get the pipeline from the cache
        let Some(ascii_pipeline) =
            pipeline_cache.get_render_pipeline(ascii_pipeline_resource.pipeline_id)
        else {
            return Ok(());
        };

        let Some(pixel_pipeline) =
            pipeline_cache.get_render_pipeline(pixel_pipeline_resource.pipeline_id)
        else {
            return Ok(());
        };

        // let acsii_camera = world.resource::<AsciiCamera>();
        // Get the settings uniform binding
        let settings_uniforms = ascii_camera.buffer(
            render_context.render_device(),
            world.get_resource::<RenderQueue>().unwrap(),
            view_target.main_texture().width(),
        );

        let Some(low_res_texture) = pixel_pipeline_resource.low_res_textures.get(&entity) else {
            return Ok(());
        };

        let Some(overlay_texture) = ascii_pipeline_resource.overlay_textures.get(&entity) else {
            return Ok(());
        };

        let overlay_texture = overlay_texture.create_view(&TextureViewDescriptor {
            label: None,
            ..Default::default()
        });

        let Some(settings_binding) = settings_uniforms.binding() else {
            return Ok(());
        };

        // This will start a new "post process write", obtaining two texture
        // views from the view target - a `source` and a `destination`.
        // `source` is the "current" main texture and you _must_ write into
        // `destination` because calling `post_process_write()` on the
        // [`ViewTarget`] will internally flip the [`ViewTarget`]'s main
        // texture to the `destination` texture. Failing to do so will cause
        // the current main texture information to be lost.
        let post_process = view_target.post_process_write();

        pixel_pass(
            low_res_texture,
            render_context,
            pixel_pipeline,
            pixel_pipeline_resource,
            &post_process,
        );

        // The bind_group gets created each frame.
        //
        // Normally, you would create a bind_group in the Queue set,
        // but this doesn't work with the post_process_write().
        // The reason it doesn't work is because each post_process_write will alternate the source/destination.
        // The only way to have the correct source/destination for the bind_group
        // is to make sure you get it during the node execution.
        let bind_group = render_context.render_device().create_bind_group(
            "post_process_bind_group",
            &ascii_pipeline_resource.layout,
            // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
            &BindGroupEntries::sequential((
                // Make sure to use the source view
                low_res_texture,
                // use the font texture
                &ascii_pipeline_resource.font_texture,
                //The overlay texture
                &overlay_texture,
                // Use the sampler created for the pipeline
                &ascii_pipeline_resource.sampler,
                // Set the settings binding
                settings_binding.clone(),
            )),
        );

        // Begin the render pass
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ascii_post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                // We need to specify the post process destination view here
                // to make sure we write to the appropriate texture.
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
        });

        // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
        // using the pipeline/bind_group created above
        render_pass.set_render_pipeline(ascii_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

fn pixel_pass(
    low_res_texture: &TextureView,
    render_context: &mut RenderContext,
    pixel_pipeline: &RenderPipeline,
    pixel_pipeline_resource: &PixelShaderPipeline,
    post_process: &PostProcessWrite,
) {
    let pixel_bind_group = render_context.render_device().create_bind_group(
        "pixel_shader_bind_group",
        &pixel_pipeline_resource.layout,
        &BindGroupEntries::sequential((post_process.source, &pixel_pipeline_resource.sampler)),
    );

    let mut pixel_render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("pixel_shader_render_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            // We need to specify the post process destination view here
            // to make sure we write to the appropriate texture.
            view: low_res_texture,
            resolve_target: None,
            ops: Operations::default(),
        })],
        depth_stencil_attachment: None,
    });

    pixel_render_pass.set_render_pipeline(pixel_pipeline);
    pixel_render_pass.set_bind_group(0, &pixel_bind_group, &[]);
    pixel_render_pass.draw(0..3, 0..1);
}

//=============================================================================
//             Extract Step
//=============================================================================

pub(crate) fn extract_camera(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &Camera, &AsciiCamera, Option<&AsciiUi>, Option<&Children>, Option<&RenderLayers>)>>,
    ui_elements: Extract<Query<(&AsciiNode, &AsciiGlobalBounds, Option<&Children>)>>,
    mut is_initialized: Local<bool>,
    mut last_surface : Local<AsciiSurface>
) {
    for (entity, camera, pixel_camera, ascii_ui, children, render_layers) in &cameras {
        if camera.is_active && pixel_camera.should_render {
            let mut entity = commands.get_or_spawn(entity);
            entity.insert(pixel_camera.clone());
            
            if let Some(render_layer) = render_layers {
                entity.insert(render_layer.clone());
            }
            
            if let Some(ascii_ui) = ascii_ui {
                // if ascii_ui.is_dirty() || !*has_rendered {
                //     let surface = AsciiSurface::new(pixel_camera.target_res().x as u32, pixel_camera.target_res().y as u32);
                //     if let Some(children) = children {
                //         for child in children.iter() {
                //             // render_ui_recursive(*child, &surface, &ui_elements);
                //         }
                //     }
                    
                //     *last_surface = surface.as_byte_vec();
                //     *has_rendered = true;
                // }
                
                if !*is_initialized {
                    *last_surface = AsciiSurface::new(pixel_camera.target_res().x as u32, pixel_camera.target_res().y as u32);
                }
                
                if ascii_ui.is_dirty() || !*is_initialized{
                    entity.insert(OverlayBuffer(last_surface.clone()));
                }
                
                *is_initialized = false;
            }
        }
    }
}

// fn render_ui_recursive(entity : Entity, surface : &AsciiSurface, nodes : &Extract<Query<(&AsciiUiNode, Option<&Children>)>>) {
//     println!("Rendering Child {:?}", entity);
//     let Ok((node, children)) = nodes.get(entity) else {return};
//     if node.bounds.width <= 0 || node.bounds.height <= 0 {return}
    
//     let buffer = AsciiBuffer::new(surface, &node.bounds);
//     node.render(&buffer);
    
//     if let Some(children) = children {
//         for child in children.iter() {
//             render_ui_recursive(*child, surface, nodes)
//         }
//     }
// }

//=============================================================================
//             Prepare Step
//=============================================================================

// Thiw will calculate the target resolution for the effect. If this resolution changes,
// it will remake the texture.
pub fn prepare_shader_textures(
    mut pixel_shader_pipeline: ResMut<PixelShaderPipeline>,
    mut ascii_shader_pipeline: ResMut<AsciiShaderPipeline>,
    acsii_cameras: Query<(Entity, &AsciiCamera, Option<&OverlayBuffer>)>,
    render_device: ResMut<RenderDevice>,
    render_queue: ResMut<RenderQueue>,
    windows: Res<ExtractedWindows>,
) {
    let window = windows.windows.get(&windows.primary.unwrap()).unwrap();

    for (entity, ascii_camera, overlay_buffer) in acsii_cameras.iter() {
        let target_resolution = ascii_camera.target_res();
        //First check to see if the render texture for the pixel shader needs updating.
        if *target_resolution != pixel_shader_pipeline.target_size
            || !pixel_shader_pipeline.low_res_textures.contains_key(&entity)
        {
            pixel_shader_pipeline.target_size = *target_resolution;
            let low_res_texture = render_device
                .create_texture(&TextureDescriptor {
                    label: "low_res_texture".into(),
                    size: Extent3d {
                        width: target_resolution.x as u32,
                        height: target_resolution.y as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::bevy_default(),
                    usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[TextureFormat::bevy_default()],
                })
                .create_view(&TextureViewDescriptor {
                    label: Some("low_res_texture"),
                    ..TextureViewDescriptor::default()
                });

            pixel_shader_pipeline
                .low_res_textures
                .insert(entity, low_res_texture);
        }

        //Then do the same thing with the overlay shaders
        if *target_resolution != ascii_shader_pipeline.target_size
            || !ascii_shader_pipeline.overlay_textures.contains_key(&entity)
        {
            ascii_shader_pipeline.target_size = *target_resolution;
            let overlay_texture = render_device.create_texture(&TextureDescriptor {
                label: "overlay_texture".into(),
                size: Extent3d {
                    width: target_resolution.x as u32,
                    height: target_resolution.y as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Uint,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });

            ascii_shader_pipeline
                .overlay_textures
                .insert(entity, overlay_texture);
        }

        //Here we need to update the overlay textures:
        if let Some(overlay_buffer) = overlay_buffer {
            if let Some(overlay_texture) = ascii_shader_pipeline.overlay_textures.get(&entity) {
                render_queue.write_texture(
                    overlay_texture.as_image_copy(),
                    &overlay_buffer.0.as_byte_vec(),
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some((target_resolution.x * 4.0) as u32),
                        rows_per_image: Some(target_resolution.y as u32),
                    },
                    Extent3d {
                        width: target_resolution.x as u32,
                        height: target_resolution.y as u32,
                        depth_or_array_layers: 1,
                    },
                )
            }
        }
    }
}
