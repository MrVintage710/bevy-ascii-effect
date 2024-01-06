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
    intensity: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec3<f32>
#endif
}

@group(0) @binding(3) var<uniform> settings: PostProcessSettings;

const TEXTURE_RESOLUTION : vec2<f32> = vec2<f32>(384.0, 192.0);
const TERMINAL_RESOLUTION : vec2<f32> = vec2<f32>(20.0, 20.0);
const CHARACTER_DIMENSIONS = vec2<f32>(24.0, 24.0);

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let CHARACTER_SIZE_UV = (CHARACTER_DIMENSIONS) / TEXTURE_RESOLUTION;
    
    let index = 2.0;
    let character_uv = vec2<f32>(
        ((index % 16.0) * CHARACTER_DIMENSIONS.x) / TEXTURE_RESOLUTION.x, 
        (floor(index / 16.0) * CHARACTER_DIMENSIONS.y) / TEXTURE_RESOLUTION.y
    );

    let output_dims = vec2<f32>(textureDimensions(screen_texture));
    let character_dims = (output_dims / TERMINAL_RESOLUTION) / output_dims;

    let fragment_value = (in.uv % character_dims) / character_dims;

    let uv = character_uv + (fragment_value * CHARACTER_SIZE_UV);
    let color = vec4<f32>(textureSample(font_texture, texture_sampler, uv));

    // if (fragment_value.x < 0.1) {
        // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    // } else if (fragment_value.x > 0.9) {
        // return vec4<f32>(0.0, 1.0, 0.0, 1.0);
    // } else {
        return color;
    // }

    // return vec4<f32>(in.uv % character_dims, 0.0, 1.0);

    //Sample each color channel with an arbitrary shift
    
    // return textureSample(font_texture, texture_sampler, in.uv);
}