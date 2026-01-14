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

    // Ring properties modulated by audio
    let ring_radius = 0.3 + bass * 0.2; // Inner radius varies with bass
    let ring_width = 0.05 + mids * 0.1; // Ring width varies with mids
    let ring_outer = ring_radius + ring_width;

    // Pulsing effect with time and highs
    let pulse = sin(time * 3.0) * 0.1 * highs;
    let inner_radius = ring_radius + pulse;
    let outer_radius = ring_outer + pulse;

    // Create ring mask
    let ring_mask = smoothstep(inner_radius - 0.01, inner_radius, dist) *
                    (1.0 - smoothstep(outer_radius - 0.01, outer_radius, dist));

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