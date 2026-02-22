struct VertexOut {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOut {
    var positions = array<vec2<f32>, 4>(
        vec2<f32>(-0.02, 0.0),
        vec2<f32>(0.02, 0.0),
        vec2<f32>(0.0, -0.02),
        vec2<f32>(0.0, 0.02),
    );

    var out: VertexOut;
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.05, 0.05, 0.05, 1.0);
}
