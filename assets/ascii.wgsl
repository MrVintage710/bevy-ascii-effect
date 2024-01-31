#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var font_texture: texture_2d<f32>;
@group(0) @binding(2) var overlay_texture: texture_2d<f32>;
@group(0) @binding(3) var texture_sampler: sampler;

struct PostProcessSettings {
    pixels_per_character: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec3<f32>
#endif
}

@group(0) @binding(4) var<uniform> settings: PostProcessSettings;

const TEXTURE_RESOLUTION : vec2<f32> = vec2<f32>(384.0, 192.0);
const CHARACTER_DIMENSIONS = vec2<f32>(24.0, 24.0);

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {

    let output_dims = vec2<f32>(textureDimensions(screen_texture));
    
    let screen_color = textureSample(screen_texture, texture_sampler, in.uv);
    let overlay_info = textureSample(overlay_texture, texture_sampler, in.uv);

    let o_index = overlay_info.x;
    
    let current_pixel = vec2<u32>(
        u32(floor(settings.pixels_per_character * (floor(in.position.x / settings.pixels_per_character)))),
        u32(floor(settings.pixels_per_character * (floor(in.position.y / settings.pixels_per_character))))
    );
    
    let value = max(screen_color.x, max(screen_color.y, screen_color.z));

    var indices = array<f32, 10>(
        46.0,
        58.0,
        45.0,
        43.0,
        42.0,
        88.0,
        87.0,
        81.0,
        86.0,
        102.0
    );

    let index = indices[i32(floor(value / 0.1))];
      
    let character_uv = vec2<f32>(
        ((index % 16.0) * CHARACTER_DIMENSIONS.x) / TEXTURE_RESOLUTION.x, 
        (floor(index / 16.0) * CHARACTER_DIMENSIONS.y) / TEXTURE_RESOLUTION.y
    );
    let character_size_uv = CHARACTER_DIMENSIONS / TEXTURE_RESOLUTION;

    let screen_pixel_uv = vec2<f32>(1.0, 1.0) / output_dims;

    // This value is 0.0 - 1.0 depending on how far along a pixel we are
    let inner_pixel_uv = (in.uv % screen_pixel_uv) / screen_pixel_uv;

    let font_uv = character_uv + (character_size_uv * inner_pixel_uv);
    
    let font_color = textureSample(font_texture, texture_sampler, font_uv);
    
    if(o_index == 0.0) {
        return vec4<f32>(1.0, 0.0, 0.0, 0.0);
    } else if (font_color.x == 1.0) {
        return screen_color;
    } else {
        return font_color;
    }
}
