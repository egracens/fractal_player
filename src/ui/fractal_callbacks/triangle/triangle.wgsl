// Uniform data: [bass, mids, highs, time]
@group(0) @binding(0)
var<uniform> audio_data: vec4<f32>;

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    let bass = audio_data.x;
    let time = audio_data.w;

    // Slow rotation (10 seconds per cycle)
    let angle = time * (6.283185 / 10.0); // 2π/10
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    let rotated = vec2<f32>(
        position.x * cos_a - position.y * sin_a,
        position.x * sin_a + position.y * cos_a
    );

    // Bass-reactive movement and SMOOTH scaling
    let movement = vec2<f32>(bass * 0.15 - 0.075, bass * 0.1);

    // JELLY-LIKE scaling - dramatic size changes, slow transitions
    let target_scale = 0.5 + bass * 1.0; // Larger range: 0.5x to 1.5x
    // Very slow smoothing for jelly-like bounce effect
    let smoothing_factor = 0.03; // Much slower transitions
    let jelly_wobble = sin(time * 0.15) * 0.02; // Slow, subtle continuous motion
    let smooth_scale = 0.8 + (target_scale - 0.8) * smoothing_factor + jelly_wobble;

    let final_pos = (rotated + movement) * smooth_scale;

    return vec4<f32>(final_pos, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    let bass = audio_data.x;
    let mids = audio_data.y;
    let highs = audio_data.z;
    let time = audio_data.w;

    // Smooth color rotation cycle (10 seconds) - continuous interpolation
    let cycle_time = 10.0;
    let phase = (time % cycle_time) / cycle_time; // 0 to 1 over 10 seconds

    // Define the three color assignments
    let colors1 = vec3<f32>(bass, mids, highs);    // Phase 0: R=bass, G=mids, B=highs
    let colors2 = vec3<f32>(highs, bass, mids);    // Phase 1: R=highs, G=bass, B=mids
    let colors3 = vec3<f32>(mids, highs, bass);    // Phase 2: R=mids, G=highs, B=bass

    // Smooth transitions between color assignments
    var final_colors: vec3<f32>;

    if (phase < 0.333) {
        // Transition from colors1 to colors2
        let t = phase / 0.333; // 0 to 1 within this segment
        let smooth_t = smoothstep(0.0, 1.0, t);
        final_colors = mix(colors1, colors2, smooth_t);
    } else if (phase < 0.666) {
        // Transition from colors2 to colors3
        let t = (phase - 0.333) / 0.333; // 0 to 1 within this segment
        let smooth_t = smoothstep(0.0, 1.0, t);
        final_colors = mix(colors2, colors3, smooth_t);
    } else {
        // Transition from colors3 back to colors1
        let t = (phase - 0.666) / 0.334; // 0 to 1 within this segment
        let smooth_t = smoothstep(0.0, 1.0, t);
        final_colors = mix(colors3, colors1, smooth_t);
    }

    let red = final_colors.x;
    let green = final_colors.y;
    let blue = final_colors.z;

    // Enhanced colors with brightness boost
    let color = vec3<f32>(
        red * 1.3 + 0.4,
        green * 1.3 + 0.4,
        blue * 1.3 + 0.4
    );

    return vec4<f32>(color, 1.0);
}
