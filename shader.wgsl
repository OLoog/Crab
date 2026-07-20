@group(0) @binding(0) var<storage, read_write> data: array<f32>;

@compute @workgroup_size(256, 1, 1)
fn mainKernel(@builtin(global_invocation_id) id: vec3<u32>) {
    var i = id.x;
    var acc: f32 = 0.0;
    loop {
        acc = acc * 1.0001 + f32(i) * 0.0001;
        if (acc > 1e9) {
            acc = 0.0;
        }
        data[i] = acc;
    }
}