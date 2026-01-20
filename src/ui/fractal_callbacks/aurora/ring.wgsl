// Uniform data: [bass, mids, highs, time]
@group(0) @binding(0)
var<uniform> audio_data: vec4<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>, // Screen coordinates for fragment shader
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate full-screen quad vertices
    var positions = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0),  // Bottom-left
        vec2<f32>(1.0, -1.0),   // Bottom-right
        vec2<f32>(-1.0, 1.0),   // Top-left
        vec2<f32>(1.0, 1.0)     // Top-right
    );

    let pos = positions[vertex_index];
    var output: VertexOutput;
    output.position = vec4<f32>(pos, 0.0, 1.0);
    output.uv = pos; // Pass screen coordinates to fragment shader
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Convert UV from [-1,1] to [0,1] range
    let uv = (input.uv + 1.0) * 0.5;

    // Center the coordinates
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(uv, center);

    // Create ring parameters based on audio data
    let bass = audio_data.x;
    let mids = audio_data.y;
    let highs = audio_data.z;
    let time = audio_data.w;

    // Calculate polar coordinates
    let offset = uv - center;
    let angle = atan2(offset.y, offset.x) + 3.14159265359; // Normalize to 0 to 2π (eliminates discontinuity at π)

    // Ring properties modulated by audio
    let base_radius = 0.3;
    let thickness = 0.05;

    // Zigzag parameters
    let num_edges = round(150.0); // Number of zigzags (8-20 edges, MUST be integer for seamless loop)
    let zigzag_amplitude = bass * 0.008; // How far zigzags push in/out (based on bass)

    // Calculate zigzag offset using sine wave
    let zigzag = sin(angle * num_edges) * zigzag_amplitude;

    // Add time-based pulsing for extra dynamics
    let pulse = sin(time * 3.0) * 0.02 * bass;

    // Apply zigzag to center radius, then add/subtract thickness
    // This maintains consistent ring width and prevents pinching
    let center_radius = base_radius + zigzag + pulse;
    let inner_radius = center_radius - thickness / 2.0;
    let outer_radius = center_radius + thickness / 2.0;

    // Create smooth ring mask (antialiased edges)
    let smoothness = 0.01;
    let ring_mask = smoothstep(inner_radius - smoothness, inner_radius + smoothness, dist) *
                    (1.0 - smoothstep(outer_radius - smoothness, outer_radius + smoothness, dist));

    // Color the ring based on audio frequencies
    let ring_color = vec3<f32>(
        bass,  // Red from bass
        mids,  // Green from mids
        highs  // Blue from highs
    );

    // Add some time-based color variation
    let hue_shift = sin(time * 2.0) * 0.3 + 0.7;
    let final_color = ring_color * hue_shift;

    // Return ring color with alpha from mask
    return vec4<f32>(final_color, ring_mask);
}
