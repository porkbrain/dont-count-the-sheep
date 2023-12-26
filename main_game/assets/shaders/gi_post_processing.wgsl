#import bevy_magic_light_2d::gi_camera::{screen_to_world, world_to_sdf_uv, bilinear_sample_rgba}
#import bevy_pbr::{
    mesh_view_bindings::view,
    forward_io::VertexOutput,
    utils::coords_to_viewport_uv,
}

@group(1) @binding(0) var in_floor_texture:              texture_2d<f32>;
@group(1) @binding(1) var in_floor_sampler:              sampler;
@group(1) @binding(6) var in_irradiance_texture:         texture_2d<f32>;
@group(1) @binding(7) var in_irradiance_texture_sampler: sampler;

fn lin_to_srgb(color: vec3<f32>) -> vec3<f32> {
   let x = color * 12.92;
   let y = 1.055 * pow(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(0.4166667)) - vec3<f32>(0.055);
   var clr = color;
   clr.x = select(x.x, y.x, (color.x < 0.0031308));
   clr.y = select(x.y, y.y, (color.y < 0.0031308));
   clr.z = select(x.z, y.z, (color.z < 0.0031308));
   return clr;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let position = in.position;
    let uv = coords_to_viewport_uv(position.xy, view.viewport);

    // Read diffuse textures.
    let in_floor_diffuse   = textureSample(in_floor_texture,   in_floor_sampler, uv);

    let in_irradiance = textureSample(in_irradiance_texture, in_irradiance_texture_sampler, uv).xyz;

    let k_size = 3;
    let k_width = 28;

    let floor_irradiance_srgb   = lin_to_srgb(in_irradiance);

    let final_floor   = in_floor_diffuse.xyz   * floor_irradiance_srgb;

    var out = vec4<f32>(final_floor, 1.0);

    return out;
}
