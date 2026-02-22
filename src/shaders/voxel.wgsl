struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) face_uv: vec2<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) face_uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = camera.view_proj * vec4<f32>(input.position, 1.0);
    out.color = input.color;
    out.face_uv = input.face_uv;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let edge_dist = min(
        min(input.face_uv.x, 1.0 - input.face_uv.x),
        min(input.face_uv.y, 1.0 - input.face_uv.y),
    );

    let outline = 1.0 - smoothstep(0.04, 0.10, edge_dist);
    let darkened = input.color * 0.55;
    let final_color = mix(input.color, darkened, outline);

    return vec4<f32>(final_color, 1.0);
}
