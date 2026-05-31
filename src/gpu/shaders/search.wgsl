@group(0) @binding(0)
var<storage, read_write> particles: array<Particle>;

@group(0) @binding(1) 
var<uniform> constants: Constants;

@group(0) @binding(2) 
var<storage, read_write> cells_ids: array<u32>;

@group(0) @binding(3) 
var<storage, read_write> particle_ids: array<u32>;

@group(0) @binding(4)
var<storage, read_write> lookups: array<Lookup>;

@group(0) @binding(5)
var<storage, read_write> predicted_pos: array<vec2<f32>>;

fn grid_coord(pos: vec2<f32>) -> vec2<u32> {
    return vec2<u32>(floor(pos / constants.cell_size));
}

fn hash(grid_coord: vec2<u32>) -> u32 {
    let cells_per_row: u32 = u32(floor(constants.width / constants.cell_size));
    return grid_coord.y * cells_per_row + grid_coord.x;
}

@compute @workgroup_size(128)
fn hash_particles(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if index >= constants.no_particles { return; }
    let clamped_pos = clamp(predicted_pos[index], vec2<f32>(0.0, 0.0), vec2<f32>(constants.width - 0.1, constants.height - 0.1));
    let grid_coord = hash(grid_coord(clamped_pos));

    cells_ids[index] = grid_coord;
    particle_ids[index] = index;
}

@compute @workgroup_size(128)
fn build_lookups(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if index >= constants.no_particles { return; }
    let cell_id = cells_ids[index];

    if index == 0u || cells_ids[index - 1u] != cell_id {
        lookups[cell_id].start_index = index;
    } if index == constants.no_particles - 1u || cells_ids[index + 1u] != cell_id {
        lookups[cell_id].end_index = index + 1u;
    }
}

