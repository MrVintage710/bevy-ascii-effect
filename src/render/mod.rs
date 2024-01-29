mod ascii;
mod pixel;

use bevy::{
    app::Plugin,
    core_pipeline::{core_3d, prepass::ViewPrepassTextures},
    prelude::*,
    render::{
        self,
        render_graph::{RenderGraphApp, ViewNode, ViewNodeRunner},
        render_resource::{
            BindGroupEntries, Extent3d, Operations, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipeline, TextureDescriptor, TextureDimension,
            TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ExtractedWindows, PostProcessWrite, ViewTarget},
        Extract, Render, RenderApp, RenderSet,
    },
};

use crate::ascii::{AsciiCamera, AsciiShaderSettingsBuffer};

use self::{ascii::AsciiShaderPipeline, pixel::PixelShaderPipeline};

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
                prepare_pixel_shader.in_set(RenderSet::PrepareResources),
            )
            .add_systems(ExtractSchedule, extract_camera)
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
    type ViewQuery = (
        Entity,
        &'static ViewTarget,
        &'static AsciiCamera,
        &'static ViewPrepassTextures,
    );

    fn run(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        view_query: bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let (entity, view_target, ascii_camera, prepass_textures) = view_query;

        let Some(depth_texture) = &prepass_textures.depth else {
            return Ok(());
        };

        // let depth_texture_view = depth_texture.texture.create_view(&TextureViewDescriptor {
        //     label: Some("depth_texture_view"),
        //     format: Some(TextureFormat::Depth32Float),
        //     dimension: Some(TextureViewDimension::D2),
        //     ..Default::default()
        // });

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
        );

        let Some(low_res_texture) = pixel_pipeline_resource.low_res_textures.get(&entity) else {
            return Ok(());
        };

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
                //The Depth Texture
                &depth_texture.default_view,
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

pub fn extract_camera(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &Camera, &AsciiCamera)>>,
) {
    for (entity, camera, pixel_camera) in &cameras {
        if camera.is_active {
            commands.get_or_spawn(entity).insert(pixel_camera.clone());
        }
    }
}

//=============================================================================
//             Prepare Step
//=============================================================================

// Thiw will calculate the target resolution for the effect. If this resolution changes,
// it will remake the texture.
pub fn prepare_pixel_shader(
    mut pixel_shader_pipeline: ResMut<PixelShaderPipeline>,
    acsii_cameras: Query<(Entity, &AsciiCamera)>,
    render_device: ResMut<RenderDevice>,
    windows: Res<ExtractedWindows>,
) {
    let window = windows.windows.get(&windows.primary.unwrap()).unwrap();

    for (entity, ascii_camera) in acsii_cameras.iter() {
        let target_resolution = Vec2::new(
            (window.physical_width as f32 / ascii_camera.pixels_per_character).floor(),
            (window.physical_height as f32 / ascii_camera.pixels_per_character).floor(),
        );

        if target_resolution != pixel_shader_pipeline.target_size
            || !pixel_shader_pipeline.low_res_textures.contains_key(&entity)
        {
            pixel_shader_pipeline.target_size = target_resolution;
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
    }
}
