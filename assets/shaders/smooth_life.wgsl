// SmoothLife compute shader
// Reads the previous frame's state from the `input` texture and writes the new state to `output`.

@group(0) @binding(0) var input: texture_storage_2d<rgba32float, read>;
@group(0) @binding(1) var output: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var<uniform> config: SmoothLifeUniforms;

struct SmoothLifeUniforms {
    alive_color: vec4<f32>,
}

const RA: i32 = 21;
const RI: i32 = 7;
const ALPHA_N: f32 = 0.028;
const ALPHA_M: f32 = 0.147;
const B1: f32 = 0.257;
const D1: f32 = 0.365;
const B2: f32 = 0.336;
const D2: f32 = 0.549;
const DT: f32 = 0.05;

fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ (state >> 16u);
    state = state * 2654435769u;
    state = state ^ (state >> 16u);
    state = state * 2654435769u;
    return state;
}

fn randomFloat(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}

fn sigma1_n(x: f32, a: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(x - a) * 4.0 / ALPHA_N));
}

fn sigma1_m(x: f32, a: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(x - a) * 4.0 / ALPHA_M));
}

fn sigma2(x: f32, a: f32, b: f32) -> f32 {
    return sigma1_n(x, a) * (1.0 - sigma1_n(x, b));
}

fn sigmamm(x: f32, y: f32, m: f32) -> f32 {
    return x * (1.0 - sigma1_m(m, 0.5)) + y * sigma1_m(m, 0.5);
}

fn s(n: f32, m: f32) -> f32 {
    return sigma2(n, sigmamm(B1, D1, m), sigmamm(B2, D2, m));
}

fn wrap_coord(v: i32, size: i32) -> i32 {
    return ((v % size) + size) % size;
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> vec3<f32> {
    let c = (1.0 - abs(2.0 * l - 1.0)) * s;
    let x = c * (1.0 - abs((h / 60.0) % 2.0 - 1.0));
    let m = l - c / 2.0;

    var rgb: vec3<f32>;
    let h_mod = i32(h / 60.0) % 6;
    if (h_mod == 0) { rgb = vec3(c, x, 0.0); }
    else if (h_mod == 1) { rgb = vec3(x, c, 0.0); }
    else if (h_mod == 2) { rgb = vec3(0.0, c, x); }
    else if (h_mod == 3) { rgb = vec3(0.0, x, c); }
    else if (h_mod == 4) { rgb = vec3(x, 0.0, c); }
    else { rgb = vec3(c, 0.0, x); }

    return rgb + vec3(m);
}

fn pixel_color(state: f32) -> vec4<f32> {
    if (state > 0.0) {
        let hue = (1.0 - state) * 240.0;
        let rgb = hsl_to_rgb(hue, 1.0, 0.5);
        // alpha 通道必须存放真实状态值，update 才能读取
        return vec4(rgb, state);
    } else {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }
}

@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let randomNumber = randomFloat((invocation_id.y << 16u) | invocation_id.x);
    let state = randomNumber;
    let color = pixel_color(state);
    textureStore(output, location, color);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let dims = vec2<i32>(i32(textureDimensions(input).x), i32(textureDimensions(input).y));

    let ra = RA - 1;

    var m_sum: f32 = 0.0;
    var m_count: f32 = 0.0;
    var n_sum: f32 = 0.0;
    var n_count: f32 = 0.0;

    for (var dy: i32 = -ra; dy <= ra; dy = dy + 1) {
        for (var dx: i32 = -ra; dx <= ra; dx = dx + 1) {
            let x = wrap_coord(location.x + dx, dims.x);
            let y = wrap_coord(location.y + dy, dims.y);
            let dist_sq = dx * dx + dy * dy;
            let val = textureLoad(input, vec2<i32>(x, y)).a;

            if (dist_sq <= RI * RI) {
                m_sum += val;
                m_count += 1.0;
            } else if (dist_sq <= RA * RA) {
                n_sum += val;
                n_count += 1.0;
            }
        }
    }

    let m = m_sum / m_count;
    let n = n_sum / n_count;

    let current = textureLoad(input, location).a;
    let diff = 2.0 * s(n, m) - 1.0;
    let new_state = clamp(current + DT * diff, 0.0, 1.0);

    let color = pixel_color(new_state);
    textureStore(output, location, color);
}
