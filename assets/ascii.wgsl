#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var font_texture: texture_2d<f32>;
@group(0) @binding(2) var overlay_texture: texture_2d<u32>;
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
    let screen_pos = vec2<u32>(floor(output_dims * in.uv));
    
    let overlay_info = textureLoad(overlay_texture, screen_pos, 0);
    
        
    let screen_color = textureSampleLevel(screen_texture, texture_sampler, in.uv, 0.0);

    let o_index = overlay_info.x;
    
    let current_pixel = vec2<u32>(
        u32(floor(settings.pixels_per_character * (floor(in.position.x / settings.pixels_per_character)))),
        u32(floor(settings.pixels_per_character * (floor(in.position.y / settings.pixels_per_character))))
    );
    
    let value = screen_color.w;
    
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
    
    var colors = array<vec3<f32>, 16>(
        vec3<f32>(0.0, 0.0, 0.0), //Black
        vec3<f32>(1.0, 1.0, 1.0), //White
        vec3<f32>(0.533, 0.0, 0.0), //Red
        vec3<f32>(0.667, 1.0, 0.933), //Cyan
        vec3<f32>(0.8, 0.267, 0.8), //Violet
        vec3<f32>(0.0, 0.8, 0.333), //Green
        vec3<f32>(0.0, 0.0, 0.667), //Blue
        vec3<f32>(0.933, 0.933, 0.467), //Yellow
        vec3<f32>(0.867, 0.533, 0.333), //Orange
        vec3<f32>(0.4, 0.267, 0.0), //Brown
        vec3<f32>(1.0, 0.467, 0.467), //Light Red
        vec3<f32>(0.2, 0.2, 0.2), //Dark Grey
        vec3<f32>(0.467, 0.467, 0.467), //Grey
        vec3<f32>(0.667, 1.0, 0.4), //Light Green
        vec3<f32>(0.0, 0.533, 1.0), //Light Blue
        vec3<f32>(0.733, 0.733, 0.733) //Light Grey
    );

    var index = indices[i32(floor(value / 0.1))];

    if(overlay_info.w == u32(1)) {
        index = f32(min(overlay_info.x, u32(127)));
    }
      
    let character_uv = vec2<f32>(
        ((index % 16.0) * CHARACTER_DIMENSIONS.x) / TEXTURE_RESOLUTION.x, 
        (floor(index / 16.0) * CHARACTER_DIMENSIONS.y) / TEXTURE_RESOLUTION.y
    );
    let character_size_uv = CHARACTER_DIMENSIONS / TEXTURE_RESOLUTION;

    let screen_pixel_uv = vec2<f32>(1.0, 1.0) / output_dims;

    // This value is 0.0 - 1.0 depending on how far along a pixel we are
    let inner_pixel_uv = (in.uv % screen_pixel_uv) / screen_pixel_uv;

    let font_uv = character_uv + (character_size_uv * inner_pixel_uv);
    
    let font_color = textureSampleLevel(font_texture, texture_sampler, font_uv, 1.0);

    if(overlay_info.w == u32(1)) {      
        if (font_color.x == 1.0) {
            var color_index = min(u32(15), overlay_info.y);
            return vec4<f32>(colors[i32(color_index)], 1.0);
        } else {
            var color_index = min(u32(15), overlay_info.z);
            return vec4<f32>(colors[i32(color_index)], 1.0);
        }
    } 
    
    if (font_color.x == 1.0) {
        return screen_color;
    } else {
        return font_color;
    }
}
