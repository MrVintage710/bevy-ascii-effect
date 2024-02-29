use crate::{
    ascii::{AsciiCamera, AsciiShaderSettingsBuffer},
    ui::buffer::AsciiSurface,
};
use bevy::{
    asset::AssetServer,
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::world::FromWorld,
    prelude::*,
    render::{
        self,
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, Extent3d, FragmentState,
            ImageCopyTexture, ImageDataLayout, MultisampleState, Origin3d, PipelineCache,
            PrimitiveState, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, ShaderType, Texture, TextureAspect, TextureFormat,
            TextureSampleType, TextureView, TextureViewDescriptor, TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{
            BevyDefault, CompressedImageFormats, Image, ImageFormat, ImageSampler, ImageType,
        },
    },
    utils::hashbrown::HashMap,
};

//=============================================================================
//             Ascii Shader Pipeline
//=============================================================================

// This contains global data used by the render pipeline. This will be created once on startup.
#[derive(Resource)]
pub(crate) struct AsciiShaderPipeline {
    pub overlay_textures: HashMap<Entity, Texture>,
    pub target_size: Vec2,
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub font_texture: TextureView,
    pub pipeline_id: CachedRenderPipelineId,
    pub overlay: Option<Vec<u8>>,
}

impl FromWorld for AsciiShaderPipeline {
    fn from_world(world: &mut World) -> Self {
        let queue = world.get_resource::<RenderQueue>().unwrap();
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
                // This is the texture for the ascii font
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                //This is the depth texture that is passed in.
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Uint,
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // The sampler that will be used to sample the screen texture
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // BindGroupLayoutEntry {
                //     binding: 4,
                //     visibility: ShaderStages::FRAGMENT,
                //     ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                //     count: None,
                // },
                // The settings uniform that will control the effect
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: bevy::render::render_resource::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(AsciiShaderSettingsBuffer::min_size()),
                    },
                    count: None,
                },
            ],
            label: Some("AsciiShaderPipeline::bind_group_layout"),
        });

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world.resource::<AssetServer>().load("ascii.wgsl");

        let font_texture = Image::from_buffer(
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", "font.png")),
            ImageType::Format(ImageFormat::Png),
            CompressedImageFormats::default(),
            true,
            ImageSampler::nearest(),
        )
        .expect("There was an error reading an internal texture.");

        let texture = render_device.create_texture(&font_texture.texture_descriptor);
        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &font_texture.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * font_texture.width()),
                rows_per_image: Some(font_texture.height()),
            },
            Extent3d {
                width: font_texture.width(),
                height: font_texture.height(),
                depth_or_array_layers: 1,
            },
        );

        let font_texture = texture.create_view(&TextureViewDescriptor {
            label: "ascii_font_texture".into(),
            ..Default::default()
        });

        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            // This will add the pipeline to the cache and queue it's creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("ascii_post_process_shader".into()),
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
                // multisample: MultisampleState {
                //     count: 4,
                //     mask: !0,
                //     alpha_to_coverage_enabled: false,
                // },
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
            });

        AsciiShaderPipeline {
            overlay_textures: HashMap::new(),
            target_size: Vec2::ZERO,
            layout,
            sampler,
            font_texture,
            pipeline_id,
            overlay: None,
        }
    }
}
//=============================================================================
//             OverlayBuffer
//=============================================================================

#[derive(Component)]
pub struct OverlayBuffer(pub AsciiSurface);
