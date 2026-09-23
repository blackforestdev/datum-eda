// Each destination word occurs once in a validated, non-overlapping plan.
@group(0) @binding(0) var<storage, read> patches: array<vec2<u32>>;
@group(0) @binding(1) var<storage, read_write> destination: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>,
        @builtin(num_workgroups) groups: vec3<u32>) {
    let index = id.x + id.y * groups.x * 64u;
    if index < arrayLength(&patches) {
        let update = patches[index];
        destination[update.x] = update.y;
    }
}
