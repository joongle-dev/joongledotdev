//== Vertex Shader ==============================

@group(0) @binding(0)
var<uniform> view_projection: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> transforms: array<mat4x4<f32>, 16>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) position: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) texcoord: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) in_position: vec3<f32>,
    @location(1) in_normal: vec3<f32>,
    @location(2) in_texcoord: vec2<f32>,
    @builtin(instance_index) instance_id: u32,
) -> VertexOutput {
    var transform = transforms[instance_id];
    var c_0 = transform[0];
    var c_1 = transform[1];
    var c_2 = transform[2];
    var s_x = length(c_0);
    var s_y = length(c_1);
    var s_z = length(c_2);
    var r_0 = c_0.xyz / s_x;
    var r_1 = c_1.xyz / s_y;
    var r_2 = c_2.xyz / s_z;
    var rotation = mat3x3<f32>(r_0, r_1, r_2);
    
    var world_pos = transform * vec4<f32>(in_position, 1.0);
    var screen_pos = view_projection * world_pos;

    var out: VertexOutput;
    out.clip_position = screen_pos;
    out.position = world_pos.xyz;
    out.normal = rotation * in_normal;
    out.texcoord = in_texcoord;
    return out;
}


//== Fragment Shader ===========================

@group(2) @binding(0)
var texture_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var sampler_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var l = normalize(vec3<f32>(0.0, 10.0, 0.0) - in.position);
    var v = normalize(vec3<f32>(0.0, 10.0, 0.0) - in.position);
    var h = normalize(l + v);
    var ndoth = dot(in.normal, h);
    var diffuse = textureSample(texture_diffuse, sampler_diffuse, in.texcoord);
    var color = ndoth * diffuse;
    return vec4<f32>(color.rgb, 1.0);
}