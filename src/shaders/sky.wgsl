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

struct SkyOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> SkyOut {
    var out: SkyOut;
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );

    let p = pos[vid];
    out.clip_position = vec4<f32>(p, 0.0, 1.0);
    out.uv = p * 0.5 + vec2<f32>(0.5, 0.5);
    return out;
}

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

fn value_noise(uv: vec2<f32>) -> f32 {
    let i = floor(uv);
    let f = fract(uv);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash(i + vec2<f32>(0.0, 0.0));
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn world_dir_to_uv(world_dir: vec3<f32>) -> vec2<f32> {
    let dir = normalize(world_dir);
    let forward = normalize(camera.camera_forward.xyz);
    let right = normalize(camera.camera_right.xyz);
    let up = normalize(camera.camera_up.xyz);

    let x_cam = dot(dir, right);
    let y_cam = dot(dir, up);
    let z_cam = dot(dir, -forward);

    let tan_half_fov_y = max(camera.proj_params.y, 1e-4);
    let aspect = max(camera.proj_params.x, 1e-4);

    let ndc_x = (x_cam / max(z_cam, 1e-4)) / (tan_half_fov_y * aspect);
    let ndc_y = (y_cam / max(z_cam, 1e-4)) / tan_half_fov_y;

    return vec2<f32>(ndc_x * 0.5 + 0.5, 0.5 - ndc_y * 0.5);
}

@fragment
fn fs_main(in: SkyOut) -> @location(0) vec4<f32> {
    let t = clamp(in.uv.y, 0.0, 1.0);
    let horizon = vec3<f32>(0.74, 0.86, 1.0);
    let zenith = vec3<f32>(0.10, 0.24, 0.54);
    var base = mix(horizon, zenith, smoothstep(0.0, 1.0, t));

    let sun_dir = normalize(-camera.sun_direction.xyz);
    let sun_pos = world_dir_to_uv(sun_dir);
    let sun_dist = distance(in.uv, sun_pos);
    let sun_core = smoothstep(0.07, 0.0, sun_dist);
    let sun_glow = smoothstep(0.28, 0.0, sun_dist) * 0.5;
    base += vec3<f32>(1.0, 0.86, 0.62) * (sun_core * 1.0 + sun_glow * 0.5);

    let cloud_uv = in.uv * vec2<f32>(7.0, 3.8) + vec2<f32>(0.0, 0.2);
    let n1 = value_noise(cloud_uv);
    let n2 = value_noise(cloud_uv * 1.9 + vec2<f32>(2.1, 5.4));
    let cloud = smoothstep(0.58, 0.8, n1 * 0.65 + n2 * 0.35);
    let cloud_fade = smoothstep(0.15, 0.85, t) * (1.0 - smoothstep(0.85, 1.0, t));
    base = mix(base, base + vec3<f32>(0.95, 0.96, 1.0) * 0.28, cloud * cloud_fade);

    return vec4<f32>(base, 1.0);
}
