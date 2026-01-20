// Uniform data: [bass, mids, highs, time]
@group(0) @binding(0)
var<uniform> audio_data: vec4<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, 1.0)
    );

    let pos = positions[vertex_index];
    var output: VertexOutput;
    output.position = vec4<f32>(pos, 0.0, 1.0);
    output.uv = pos; 
    return output;
}

// Complex number multiplication: (a + bi) * (c + di) = (ac - bd) + (ad + bc)i
fn complex_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        a.x * b.x - a.y * b.y,
        a.x * b.y + a.y * b.x
    );
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Inputs from Time
    let time = audio_data.w;
    
    // 2. Julia Set Constant Modulation (Time-based only)
    // Base center: (-0.8, 0.156)
    let orbit_radius = 0.05;
    let orbit_speed = time * 0.5;
    let c = vec2<f32>(
        -0.8 + cos(orbit_speed) * orbit_radius,
        0.156 + sin(orbit_speed * 0.7) * orbit_radius
    );
    
    // 3. Starting Z is the UV coordinate
    // Map UV from [-1, 1] to a comfortable viewing range [-1.5, 1.5]
    // Add a slight rotation over time for more dynamism
    let rot = time * 0.1;
    let cos_r = cos(rot);
    let sin_r = sin(rot);
    let uv_rot = vec2<f32>(
        input.uv.x * cos_r - input.uv.y * sin_r,
        input.uv.x * sin_r + input.uv.y * cos_r
    );
    var z = uv_rot * 1.5;
    
    // 4. Iteration Loop
    let max_iterations = 128u;
    var iterations = 0u;
    var mag_sq = 0.0;
    
    for (iterations = 0u; iterations < max_iterations; iterations++) {
        mag_sq = dot(z, z);
        if (mag_sq > 4.0) {
            break;
        }
        
        // z = z^2 + c
        z = complex_mul(z, z) + c;
    }
    
    // 5. Coloring (Time-based only)
    if (iterations == max_iterations) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Inside the set
    } else {
        // Smooth coloring
        let log_zn = log(mag_sq) / 2.0;
        let smooth_iterations = f32(iterations) + 1.0 - log(log_zn) / log(2.0);
        let t = smooth_iterations / f32(max_iterations);
        
        // Use time to shift the colors slightly
        let r = sin(t * 10.0 + time) * 0.5 + 0.5;
        let g = sin(t * 10.0 + time * 0.7 + 2.0) * 0.5 + 0.5;
        let b = sin(t * 10.0 + time * 0.5 + 4.0) * 0.5 + 0.5;
        
        return vec4<f32>(r, g, b, 1.0);
    }
}
