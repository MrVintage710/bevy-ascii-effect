use bevy::{app::Plugin, prelude::{App, World}, ecs::{component::Component, system::Resource, world::FromWorld}, render::{extract_component::{ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin, ComponentUniforms}, render_resource::{ShaderType, BindGroupLayoutEntry, BindGroupLayoutDescriptor, ShaderStages, BindingType, TextureSampleType, TextureViewDimension, BindGroupLayout, SamplerBindingType, SamplerDescriptor, PipelineCache, RenderPipelineDescriptor, FragmentState, ColorTargetState, TextureFormat, ColorWrites, PrimitiveState, MultisampleState, Sampler, CachedRenderPipelineId, BindGroupEntries, RenderPassColorAttachment, Operations, RenderPassDescriptor}, render_graph::{ViewNode, RenderGraphApp, ViewNodeRunner}, RenderApp, renderer::RenderDevice, view::ViewTarget, texture::BevyDefault}, core_pipeline::{core_3d, fullscreen_vertex_shader::fullscreen_shader_vertex_state}, asset::AssetServer};

pub struct AsciiShaderPlugin;

//This plugin will add the settings required for the AsciiShader and add the post precess shader to the right spot on the render graph.
impl Plugin for AsciiShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<AsciiShaderSettings>::default(),
            UniformComponentPlugin::<AsciiShaderSettings>::default(),
        ));
        
        // We need to get the render app from the main app
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        
        render_app
        .add_render_graph_node::<ViewNodeRunner<AsciiShaderNode>>(
            core_3d::graph::NAME, 
            AsciiShaderNode::NAME
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
            .init_resource::<AsciiShaderPipeline>();
    }
}

//=============================================================================
//             Ascii Shader Node
//=============================================================================

#[derive(Default)]
pub struct AsciiShaderNode;
impl AsciiShaderNode {
    const NAME : &'static str = "AsciiShaderNode";
}

impl ViewNode for AsciiShaderNode {
    type ViewQuery = &'static ViewTarget;

    fn run(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        view_query: bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        
        // Get the pipeline resource that contains the global data we need
        // to create the render pipeline
        let post_process_pipeline = world.resource::<AsciiShaderPipeline>();
        
        // The pipeline cache is a cache of all previously created pipelines.
        // It is required to avoid creating a new pipeline each frame,
        // which is expensive due to shader compilation.
        let pipeline_cache = world.resource::<PipelineCache>();
        
        // Get the pipeline from the cache
        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
        else {
            return Ok(());
        };
        
        // Get the settings uniform binding
        let settings_uniforms = world.resource::<ComponentUniforms<AsciiShaderSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };
        
        // This will start a new "post process write", obtaining two texture
        // views from the view target - a `source` and a `destination`.
        // `source` is the "current" main texture and you _must_ write into
        // `destination` because calling `post_process_write()` on the
        // [`ViewTarget`] will internally flip the [`ViewTarget`]'s main
        // texture to the `destination` texture. Failing to do so will cause
        // the current main texture information to be lost.
        let post_process = view_query.post_process_write();
        
        // The bind_group gets created each frame.
        //
        // Normally, you would create a bind_group in the Queue set,
        // but this doesn't work with the post_process_write().
        // The reason it doesn't work is because each post_process_write will alternate the source/destination.
        // The only way to have the correct source/destination for the bind_group
        // is to make sure you get it during the node execution.
        let bind_group = render_context.render_device().create_bind_group(
            "post_process_bind_group",
            &post_process_pipeline.layout,
            // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
            &BindGroupEntries::sequential((
                // Make sure to use the source view
                post_process.source,
                // Use the sampler created for the pipeline
                &post_process_pipeline.sampler,
                // Set the settings binding
                settings_binding.clone(),
            )),
        );
        
        // Begin the render pass
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("post_process_pass"),
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
        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

//=============================================================================
//             Ascii Shader Pipeline
//=============================================================================

// This contains global data used by the render pipeline. This will be created once on startup.
#[derive(Resource)]
struct AsciiShaderPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for AsciiShaderPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        
        //We need to create the bind group
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                //This is the screen texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // The sampler that will be used to sample the screen texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // The settings uniform that will control the effect
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: bevy::render::render_resource::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(AsciiShaderSettings::min_size()),
                    },
                    count: None,
                },
            ],
            label: Some("AsciiShaderPipeline::bind_group_layout"),
        });
    
        // We can create the sampler here since it won't change at runtime and doesn't depend on the view
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());
        
        let shader = world
            .resource::<AssetServer>()
            .load("ascii.wgsl");
        
        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            // This will add the pipeline to the cache and queue it's creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("post_process_pipeline".into()),
                layout: vec![layout.clone()],
                // This will setup a fullscreen triangle for the vertex state
                vertex: fullscreen_shader_vertex_state(),
                fragment: Some(FragmentState {
                    shader,
                    shader_defs: vec![],
                    // Make sure this matches the entry point of your shader.
                    // It can be anything as long as it matches here and in the shader.
                    entry_point: "fragment".into(),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::bevy_default(),
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // All of the following properties are not important for this effect so just use the default values.
                // This struct doesn't have the Default trait implemented because not all field can have a default value.
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
            });
        
        AsciiShaderPipeline {
            layout,
            sampler,
            pipeline_id,
        }
    }
}

//=============================================================================
//             Shader Settings
//=============================================================================

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub struct AsciiShaderSettings {
    pub intensity: f32,
    // WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl2")]
    pub _webgl2_padding: Vec3,
}