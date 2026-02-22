struct Camera {
    view_proj: mat4x4<f32>,
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
    return smoothstep(0.02, 0.08, edge_dist);
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(-0.5, -1.0, -0.35));
    let normal = normalize(input.normal);

    let diffuse = max(dot(normal, -light_dir), 0.0);
    let ambient = 0.35;

    let height_shadow = clamp(0.55 + input.world_pos.y * 0.03, 0.55, 1.0);
    let light = (ambient + diffuse * 0.65) * height_shadow;

    let outline = edge_outline(input.world_pos, normal);
    let outlined_light = light * mix(0.4, 1.0, outline);

    return vec4<f32>(input.color * outlined_light, 1.0);
}
