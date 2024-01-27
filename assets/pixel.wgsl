
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;


@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
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

    let base_color = textureSample(screen_texture, texture_sampler, in.uv);
        
    var closest_color_index = 0;
    var value = 0.0;
    var closest_color_distance = 1.4142135623731;

    for(var i : i32 = 0; i < 16; i++) {
        let current_color = colors[i];

        let color_distance = distance(base_color.xyz, current_color);

        if(color_distance < closest_color_distance) {
            closest_color_distance = color_distance;
            closest_color_index = i;
            let curent_value = max(current_color.x, max(current_color.y, current_color.z));
            let base_value = max(base_color.x, max(base_color.y, base_color.z));
            value = distance(curent_value, base_value);
        }
    }

    // return vec4<f32>(colors[closest_color_index], value / 1.4142135623731);
    return textureSample(screen_texture, texture_sampler, in.uv);
}
