// This shader computes the chromatic aberration effect

// Since post processing is a fullscreen effect, we use the fullscreen vertex shader provided by bevy.
// This will import a vertex shader that renders a single fullscreen triangle.
//
// A fullscreen triangle is a single triangle that covers the entire screen.
// The box in the top left in that diagram is the screen. The 4 x are the corner of the screen
//
// Y axis
//  1 |  x-----x......
//  0 |  |  s  |  . ´
// -1 |  x_____x´
// -2 |  :  .´
// -3 |  :´
//    +---------------  X axis
//      -1  0  1  2  3
//
// As you can see, the triangle ends up bigger than the screen.
//
// You don't need to worry about this too much since bevy will compute the correct UVs for you.
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct PostProcessSettings {
    intensity: f32,
    time: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec3<f32>
#endif
}
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

fn remap(val: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let t = (val - in_min) / (in_max - in_min);
    return mix(out_min, out_max, clamp(t, 0.0, 1.0));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // ── CRT scanlines (always active) ─────────────────────────────────────
    // Two sine waves at different frequencies/speeds, remapped to 0.9–1.0
    // so they never go fully black — just dim bands.
    let scan_a = remap(sin(uv.y * 800.0 + settings.time * 10.0),  -1.0, 1.0, 0.70, 1.0);
    let scan_b = remap(sin(uv.y * 400.0 - settings.time * 20.0),  -1.0, 1.0, 0.80, 1.0);
    let scanline = scan_a * scan_b;

    // ── Vignette (always active) ───────────────────────────────────────────
    let centered = uv - vec2<f32>(0.5, 0.5);
    // dot(c,c) peaks at 0.5 in the corners; multiply by ~1.5 for a moderate darkening
    let vignette = clamp(1.0 - dot(centered, centered) * 1.5, 0.0, 1.0);

    // ── Chromatic aberration (intensity-gated) ─────────────────────────────
    // When intensity == 0.0 all three samples hit the same UV — zero cost.
    let off = settings.intensity;
    let r = textureSample(screen_texture, texture_sampler, uv + vec2<f32>( off, -off)).r;
    let g = textureSample(screen_texture, texture_sampler, uv + vec2<f32>(-off,  0.0)).g;
    let b = textureSample(screen_texture, texture_sampler, uv + vec2<f32>( 0.0,  off)).b;

    let color = vec3<f32>(r, g, b) * scanline * vignette;
    return vec4<f32>(color, 1.0);
}
