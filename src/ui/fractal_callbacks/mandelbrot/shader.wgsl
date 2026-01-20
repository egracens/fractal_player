// Uniform data: [bass, mids, highs, time]
@group(0) @binding(0)
var<uniform> audio_data: vec4<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) c: vec2<f32>, // Complex number (real, imaginary)
    @location(1) pixel_size: f32, // Size of one pixel in complex plane
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
    let time = audio_data.w;
    
    // Infinite zoom parameters
    let zoom_speed = 0.3;  // How fast we zoom in
    let zoom_scale = exp(-time * zoom_speed);  // Exponential zoom (gets smaller over time)
    
    // Interesting point to zoom into (edge of the Mandelbrot set)
    // This is a point on the boundary with beautiful detail
    let zoom_center = vec2<f32>(-0.7463, 0.1102);
    
    // Map UV to complex plane with zoom
    // c = pos * scale + center
    let c = pos * zoom_scale + zoom_center;
    
    // Calculate pixel size in complex plane
    // pos ranges from -1 to 1 (width of 2 in clip space)
    // We map this to zoom_scale * 2 in complex plane
    // So one pixel = (zoom_scale * 2) / screen_width_in_pixels
    // Approximating screen width as 1920 pixels
    let pixel_size_complex = (zoom_scale * 2.0) / 1920.0;
    
    var output: VertexOutput;
    output.position = vec4<f32>(pos, 0.0, 1.0);
    output.c = c;
    output.pixel_size = pixel_size_complex;
    return output;
}

// Complex number multiplication: (a + bi) * (c + di) = (ac - bd) + (ad + bc)i
fn complex_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        a.x * b.x - a.y * b.y,  // Real part
        a.x * b.y + a.y * b.x   // Imaginary part
    );
}

// Complex number squared: z² = z * z
fn complex_square(z: vec2<f32>) -> vec2<f32> {
    return complex_mul(z, z);
}

// Magnitude squared: |z|² = real² + imag²
fn magnitude_squared(z: vec2<f32>) -> f32 {
    return z.x * z.x + z.y * z.y;
}

// Calculate color for a single point in the Mandelbrot set
fn mandelbrot_color(c: vec2<f32>) -> vec4<f32> {
    // Mandelbrot iteration
    var z = vec2<f32>(0.0, 0.0);  // Start at origin
    let max_iterations = 256u;
    var iterations = 0u;
    var z_magnitude_squared = 0.0;

    for (iterations = 0u; iterations < max_iterations; iterations++) {
        z_magnitude_squared = magnitude_squared(z);
        
        // Check if escaped (|z|² > 4 means |z| > 2)
        if (z_magnitude_squared > 4.0) {
            break;
        }
        
        // z = z² + c
        z = complex_square(z) + c;
    }

    var point_in_mandelbrot_set = iterations == max_iterations;
    if (point_in_mandelbrot_set) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else {
        // Smooth coloring using continuous iteration count
        // Formula: smooth = iterations + 1 - log(log(|z|)) / log(2)
        let log_zn = log(z_magnitude_squared) / 2.0;  // log(|z|) = log(sqrt(z²)) = log(z²)/2
        let smooth_iterations = f32(iterations) + 1.0 - log(log_zn) / log(2.0);
        
        // Deep space color palette (dark blues/purples)
        let t = smooth_iterations / f32(max_iterations);
        
        // Use multiple sine waves with different frequencies for depth
        let s1 = sin(t * 3.14159);  // Half cycle
        let s2 = sin(t * 6.28318);  // Full cycle
        
        // Dark blue to purple gradient
        let r = s1 * 0.15 + s2 * 0.1;  // Dark red (0.0 - 0.25)
        let g = s1 * 0.05 + 0.02;       // Very little green (0.0 - 0.07)
        let b = s1 * 0.3 + s2 * 0.2 + 0.2;  // Dominant blue (0.2 - 0.7)
        
        return vec4<f32>(r, g, b, 1.0);
    }
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let c = input.c;  // Complex number from vertex shader (center of pixel)
    
    // No anti-aliasing - single sample per pixel
    return mandelbrot_color(c);
}
