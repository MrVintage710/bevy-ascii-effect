use bevy::{
    asset::AssetServer,
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::world::FromWorld,
    prelude::*,
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
            PipelineCache, PrimitiveState, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, TextureFormat, TextureSampleType, TextureView,
            TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
    },
    utils::HashMap,
};

//=============================================================================
//             Pixel Shader Pipeline
//=============================================================================

#[derive(Resource)]
pub(crate) struct PixelShaderPipeline {
    pub low_res_textures: HashMap<Entity, TextureView>,
    pub target_size: Vec2,
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for PixelShaderPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

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
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("downsize_shader_bind_group_layout"),
        });

        let shader = world.resource::<AssetServer>().load("pixel.wgsl");

        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            // This will add the pipeline to the cache and queue it's creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("downsize_shader_pipeline".into()),
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

        PixelShaderPipeline {
            low_res_textures: HashMap::new(),
            target_size: Vec2 { x: 1.0, y: 1.0 },
            layout,
            sampler,
            pipeline_id,
        }
    }
}
