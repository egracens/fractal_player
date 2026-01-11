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

    // Bass-reactive movement and scaling
    let movement = vec2<f32>(bass * 0.15 - 0.075, bass * 0.1);
    let scale = 0.7 + bass * 0.6;
    let final_pos = (rotated + movement) * scale;

    return vec4<f32>(final_pos, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    let bass = audio_data.x;
    let mids = audio_data.y;
    let highs = audio_data.z;
    let time = audio_data.w;

    // Color rotation cycle (10 seconds)
    let phase = (time % 10.0) / 10.0;
    var red: f32;
    var green: f32;
    var blue: f32;

    if (phase < 0.333) {
        red = bass; green = mids; blue = highs;
    } else if (phase < 0.666) {
        red = highs; green = bass; blue = mids;
    } else {
        red = mids; green = highs; blue = bass;
    }

    // Enhanced colors with brightness boost
    let color = vec3<f32>(
        red * 1.3 + 0.4,
        green * 1.3 + 0.4,
        blue * 1.3 + 0.4
    );

    return vec4<f32>(color, 1.0);
}
