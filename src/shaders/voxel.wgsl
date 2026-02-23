struct Camera {
    view_proj: mat4x4<f32>,
    sun_direction: vec4<f32>,
    camera_forward: vec4<f32>,
    camera_right: vec4<f32>,
    camera_up: vec4<f32>,
    proj_params: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

@vertex
fn vs_main(input: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = camera.view_proj * vec4<f32>(input.position, 1.0);
    out.color = input.color;
    out.world_pos = input.position;
    out.normal = normalize(input.normal);
    return out;
}

fn edge_outline(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let p = fract(world_pos);
    let an = abs(normal);

    var uv = vec2<f32>(p.x, p.y);
    if an.x > 0.5 {
        uv = vec2<f32>(p.y, p.z);
    } else if an.y > 0.5 {
        uv = vec2<f32>(p.x, p.z);
    } else {
        uv = vec2<f32>(p.x, p.y);
    }

    let edge_dist = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    return smoothstep(0.016, 0.085, edge_dist);
}

fn tri_planar_variation(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let blend = pow(abs(normal), vec3<f32>(2.2));
    let w = blend / (blend.x + blend.y + blend.z + 1e-5);

    let uvx = world_pos.yz;
    let uvy = world_pos.xz;
    let uvz = world_pos.xy;

    let nx = sin(uvx.x * 8.5 + uvx.y * 7.3) * 0.5 + 0.5;
    let ny = sin(uvy.x * 7.1 + uvy.y * 9.4) * 0.5 + 0.5;
    let nz = sin(uvz.x * 6.7 + uvz.y * 8.8) * 0.5 + 0.5;

    return nx * w.x + ny * w.y + nz * w.z;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let normal = normalize(input.normal);
    let light_dir = normalize(camera.sun_direction.xyz);
    let view_dir = normalize(camera.camera_forward.xyz);

    let diffuse = max(dot(normal, -light_dir), 0.0);
    let half_vec = normalize(-light_dir + view_dir);
    let specular = pow(max(dot(normal, half_vec), 0.0), 22.0) * 0.12;

    let ambient_sky = vec3<f32>(0.26, 0.32, 0.45);
    let warm_bounce = vec3<f32>(0.18, 0.13, 0.09) * max(-normal.y, 0.0);
    let ambient = ambient_sky * (0.45 + 0.55 * max(normal.y, 0.0)) + warm_bounce;

    let sun_shadow = 0.25 + 0.75 * diffuse;

    let height_fade = clamp(0.5 + input.world_pos.y * 0.03, 0.5, 1.1);

    let variation = tri_planar_variation(input.world_pos, normal);
    let albedo = input.color * (0.9 + 0.18 * (variation - 0.5));

    let edge = edge_outline(input.world_pos, normal);
    let edge_darkening = mix(0.62, 1.0, edge);

    let lit = albedo * (ambient + vec3<f32>(0.82 * sun_shadow)) * height_fade;
    let final_color = lit * edge_darkening + specular;

    return vec4<f32>(final_color, 1.0);
}
