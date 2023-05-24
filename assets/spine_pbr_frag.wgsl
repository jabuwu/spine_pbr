struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(10) dark_color: vec4<f32>,
    @location(11) rotation: vec4<f32>,
};

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(2)
var normal_texture: texture_2d<f32>;
@group(1) @binding(3)
var normal_texture_sampler: sampler;

@group(1) @binding(4)
var<uniform> light_position: vec4<f32>;

fn linear_to_nonlinear(x: f32) -> f32 {
    if x <= 0.0 {
        return x;
    }
    if x <= 0.0031308 {
        return x * 12.92;
    } else {
        return (1.055 * pow(x, 1.0 / 2.4)) - 0.055;
    }
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture, texture_sampler, input.uv);

    let rotation = mat3x3(
        input.rotation[0], input.rotation[1], 0.0,
        input.rotation[2], input.rotation[3], 0.0,
        0.0, 0.0, 1.0,
    );

    let normal = textureSample(normal_texture, normal_texture_sampler, input.uv) * vec4(1.0, 1.0, 1.0, 1.0);
    let normal_xyz = rotation * normalize(vec3(
        linear_to_nonlinear(normal.x),
        linear_to_nonlinear(normal.y),
        linear_to_nonlinear(normal.z)
    ) * vec3(2.0) - vec3(1.0));

    let surface_to_light = normalize(light_position.xyz - input.world_position.xyz);
    let distance = max(distance(light_position.xyz, input.world_position.xyz) / 100.0, 0.01);
    let surface_dot = clamp(dot(normal_xyz, surface_to_light), 0.05, 1.0);

    let light = clamp((surface_dot * (1.0 / distance)) * 5.0, 0.0, 1.0);

    return vec4(tex_color.rgb * light, tex_color.a);
}
