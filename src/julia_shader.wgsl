struct VertexOut {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct UniformParams {
    x_range: vec2<f32>,
    y_range: vec2<f32>,
    max_iterations: u32,
    c: vec2<f32>,
    palette: array<vec4<f32>, 128>,
};

@group(0) @binding(0)
var<uniform> uniforms: UniformParams;

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VertexOut {
    var out: VertexOut;
    out.uv = position;
    let width = (uniforms.x_range[1] - uniforms.x_range[0]);
    let height = (uniforms.y_range[1] - uniforms.y_range[0]);
    let x = mix(-1.0, 1.0, (position.x - uniforms.x_range[0]) / width);
    let y = mix(-1.0, 1.0, (position.y - uniforms.y_range[0]) / height);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
//    let x = in.uv.x;
//    let y = in.uv.y;
    var iterations = 0u;
    var escaped = false;
    var z = in.uv;
    for (var i = 0u; i < uniforms.max_iterations; i++) {
        if (z.x * z.x + z.y * z.y > 4.0) {
            escaped = true;
            iterations = i + 1u;
            break;
        }
        z= iter(z, uniforms.c);
    }
    if (!escaped) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return color(iterations);
}

fn iter(z: vec2<f32>, c: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(z.x * z.x - z.y * z.y + c.x, 2.0 * z.x * z.y + c.y);
}

fn color(iterations: u32) -> vec4<f32> {
//    let i = f32(iterations) / f32(uniforms.max_iterations);
//    let x = i * 256.0 * 256.0 * 256.0;
//    let r = x % 256.0;
//    let g = (x - r) / 256.0 % 256.0;
//    let b = (x - r - g * 256.0) / 256.0 / 256.0 % 256.0;
    let index = iterations % 128u;
    return uniforms.palette[index];
//    let c = i * 256.0;
//    return vec4<f32>(c, c, c, 1.0);
}