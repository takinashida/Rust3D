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

@fragment
fn fs_main(in: SkyOut) -> @location(0) vec4<f32> {
    let t = clamp(in.uv.y, 0.0, 1.0);
    let horizon = vec3<f32>(0.58, 0.79, 0.98);
    let zenith = vec3<f32>(0.12, 0.30, 0.66);
    let base = mix(horizon, zenith, smoothstep(0.0, 1.0, t));

    let sun_pos = vec2<f32>(0.75, 0.8);
    let sun_dist = distance(in.uv, sun_pos);
    let sun = smoothstep(0.12, 0.0, sun_dist) * 0.35;

    return vec4<f32>(base + sun, 1.0);
}
