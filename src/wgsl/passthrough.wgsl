[[stage(vertex)]]
fn main([[location(0)]] p: vec2<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(p.x, p.y, 0.0, 1.0);
}
