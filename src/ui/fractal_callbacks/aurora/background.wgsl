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
    return vec4<f32>(min(input.uv.x, 1.0), min(input.uv.y, 1.0), 1.0, 1.0);
}
