// This shader computes the chromatic aberration effect

// Since post processing is a fullscreen effect, we use the fullscreen vertex shader provided by bevy.
// This will import a vertex shader that renders a single fullscreen triangle.
//
// A fullscreen triangle is a single triangle that covers the entire screen.
// The box in the top left in that diagram is the screen. The 4 x are the corner of the screen
//
// Y axis
//  1 |  x-----x......
//  0 |  |  s  |  . ´
// -1 |  x_____x´
// -2 |  :  .´
// -3 |  :´
//    +---------------  X axis
//      -1  0  1  2  3
//
// As you can see, the triangle ends up bigger than the screen.
//
// You don't need to worry about this too much since bevy will compute the correct UVs for you.
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var font_texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

struct PostProcessSettings {
    pixels_per_character: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec3<f32>
#endif
}

@group(0) @binding(3) var<uniform> settings: PostProcessSettings;

const TEXTURE_RESOLUTION : vec2<f32> = vec2<f32>(384.0, 192.0);
const CHARACTER_DIMENSIONS = vec2<f32>(24.0, 24.0);

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {

    let index = 65.0;
    let character_uv = vec2<f32>(
        ((index % 16.0) * CHARACTER_DIMENSIONS.x) / TEXTURE_RESOLUTION.x, 
        (floor(index / 16.0) * CHARACTER_DIMENSIONS.y) / TEXTURE_RESOLUTION.y
    );
    let character_size_uv = CHARACTER_DIMENSIONS / TEXTURE_RESOLUTION;

    let output_dims = vec2<f32>(textureDimensions(screen_texture));
    let screen_pixel_uv = vec2<f32>(1.0, 1.0) / output_dims;

    // This value is 0.0 - 1.0 depending on how far along a pixel we are
    let inner_pixel_uv = (in.uv % screen_pixel_uv) / screen_pixel_uv;

    let font_uv = character_uv + (character_size_uv * inner_pixel_uv);
    
    let font_color = textureSample(font_texture, texture_sampler, font_uv);
    let screen_color = textureSample(screen_texture, texture_sampler, in.uv);

    if (font_color.x == 1.0) {
        return screen_color;
    } else {
        return font_color;
    }

    //let uv = in.uv * vec2<f32>(0.0625, 0.125);
    // let color = vec4<f32>(in.uv.x * 0.5, 0.0, 0.0, 255.0);
    // let color = vec4<f32>(textureSample(font_texture, texture_sampler, uv));

    // return color;
    // return textureSample(screen_texture, texture_sampler, in.uv);
}
