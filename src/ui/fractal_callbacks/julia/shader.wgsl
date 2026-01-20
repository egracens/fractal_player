@group(0) @binding(0)
var<uniform> audio_data: vec4<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct JuliaResult {
    iterations: u32,
    final_mag_sq: f32,
    final_z: vec2<f32>,
    max_iterations: u32,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, 1.0)
    );

    let pos = positions[vertex_index];
    var output: VertexOutput;
    output.position = vec4<f32>(pos, 0.0, 1.0);
    output.uv = pos; 
    return output;
}

fn complex_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

fn get_animated_c(time: f32) -> vec2<f32> {
    let anchor_x = -0.8;
    let anchor_y = 0.156;
    let movement_range = 0.15;
    
    let speed_x = 0.2;
    let speed_y = 0.4;
    
    return vec2<f32>(
        anchor_x + cos(time * speed_x) * movement_range,
        anchor_y + sin(time * speed_y) * movement_range
    );
}

fn spin(uv: vec2<f32>, time: f32) -> vec2<f32> {
    let rotation_speed = 0.1;
    let angle = time * rotation_speed;
    
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    
    return vec2<f32>(
        uv.x * cos_a - uv.y * sin_a,
        uv.x * sin_a + uv.y * cos_a
    );
}

fn zoom(uv: vec2<f32>, level: f32) -> vec2<f32> {
    return uv * level;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let time = audio_data.w;
    
    let c = get_animated_c(time);
    
    let rotated_uv = spin(input.uv, time);
    let view_scale = 1.5;
    let z_start = zoom(rotated_uv, view_scale);
    
    let result = iterate_julia(z_start, c);
    
    if (result.iterations == result.max_iterations) {
        return paint_attracted(result, time);
    } else {
        return paint_escaped(result, time);
    }
}

fn iterate_julia(z_start: vec2<f32>, c: vec2<f32>) -> JuliaResult {
    let max_iter = 128u;
    var z = z_start;
    var iterations = 0u;
    var mag_sq = 0.0;
    
    for (iterations = 0u; iterations < max_iter; iterations++) {
        mag_sq = dot(z, z);
        if (mag_sq > 4.0) { break; }
        z = complex_mul(z, z) + c;
    }
    
    return JuliaResult(iterations, mag_sq, z, max_iter);
}

fn paint_attracted(res: JuliaResult, time: f32) -> vec4<f32> {
    let dist = length(res.final_z);
    let r = sin(dist * 10.0 + time) * 0.1 + 0.1;
    let g = sin(dist * 15.0 + time * 0.7) * 0.05 + 0.05;
    let b = sin(dist * 5.0 + time * 0.5) * 0.2 + 0.1;
    return vec4<f32>(r, g, b, 1.0);
}

fn paint_escaped(res: JuliaResult, time: f32) -> vec4<f32> {
    let log_zn = log(res.final_mag_sq) / 2.0;
    let smooth_iter = f32(res.iterations) + 1.0 - log(log_zn) / log(2.0);
    let t = smooth_iter / f32(res.max_iterations);
    
    let r = sin(t * 10.0 + time) * 0.5 + 0.5;
    let g = sin(t * 10.0 + time * 0.7 + 2.0) * 0.5 + 0.5;
    let b = sin(t * 10.0 + time * 0.5 + 4.0) * 0.5 + 0.5;
    return vec4<f32>(r, g, b, 1.0);
}
