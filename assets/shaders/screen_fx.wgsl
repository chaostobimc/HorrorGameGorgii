// Film grain, vignette, rain streaks and lightning flash, drawn as a single
// fullscreen UI layer on top of the 3D view.
#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0) var<uniform> params: vec4<f32>;   // x grain, y vignette, z flash, w rain
@group(1) @binding(1) var<uniform> clock: vec4<f32>;    // x time
@group(1) @binding(2) var noise_tex: texture_2d<f32>;
@group(1) @binding(3) var noise_samp: sampler;
@group(1) @binding(4) var streak_tex: texture_2d<f32>;
@group(1) @binding(5) var streak_samp: sampler;

fn hash21(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(123.34, 456.21));
    q += dot(q, q + 45.32);
    return fract(q.x * q.y);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t = clock.x;

    // --- vignette -----------------------------------------------------------
    let centred = (uv - vec2<f32>(0.5, 0.5)) * vec2<f32>(1.0, 1.15);
    let d = length(centred) * 1.45;
    let vig = pow(smoothstep(0.25, 1.05, d), 1.6) * params.y;

    // --- film grain ---------------------------------------------------------
    let n1 = textureSample(noise_tex, noise_samp, uv * vec2<f32>(4.1, 3.7) + vec2<f32>(t * 0.53, t * 0.29)).r;
    let n2 = textureSample(noise_tex, noise_samp, uv * vec2<f32>(11.3, 9.1) - vec2<f32>(t * 0.91, t * 0.61)).r;
    let grain = (n1 * 0.62 + n2 * 0.38 - 0.5) * params.x;

    // --- rain on the "lens" -------------------------------------------------
    let s1 = textureSample(streak_tex, streak_samp, vec2<f32>(uv.x * 5.0, uv.y * 1.4 - t * 0.85)).r;
    let s2 = textureSample(streak_tex, streak_samp, vec2<f32>(uv.x * 9.0 + 0.37, uv.y * 2.2 - t * 1.55)).r;
    let streaks = clamp((s1 + s2) * 0.6, 0.0, 1.0) * params.w;

    // --- lightning flash ----------------------------------------------------
    let flash = clamp(params.z, 0.0, 1.0);

    let light_add = clamp(streaks * 0.55 + flash, 0.0, 1.0);
    let alpha = clamp(vig * 0.92 + light_add * 0.85 + abs(grain) * 0.9, 0.0, 1.0);
    var rgb = mix(vec3<f32>(0.005, 0.007, 0.010), vec3<f32>(0.92, 0.95, 1.0), light_add);
    rgb = rgb + vec3<f32>(grain * 0.55);
    return vec4<f32>(max(rgb, vec3<f32>(0.0)), alpha);
}
