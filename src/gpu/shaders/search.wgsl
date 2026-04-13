
struct Constants {
	width:f32,
	height:f32,
	no_particles:u32,
    max_vel: f32,
    radius: f32,
    mass: f32,
    rest_density: f32,
	dt:f32,
    gravity: vec2<f32>,
    gas_constant: f32,
    influence_radius: f32,
    cell_size: f32,
    damping: f32,
	_padding: vec2<f32>
}
struct Cell {
    cell_id: u32,
    particle_id: u32,
}

struct Lookup {
	start_index: u32,	
	count: atomic<u32>
}

struct Particle{
    pos:vec2<f32>,
    predicted_pos:vec2<f32>,
    vel: vec2<f32>,
    force: vec2<f32>,
    density:f32,
    pressure:f32,
}

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

fn grid_coord(pos: vec2<f32>) -> vec2<u32> {
	return vec2<u32>(floor(pos / constants.cell_size));
}

fn hash(grid_coord:vec2<u32>) -> u32 {
	let cells_per_row: u32 = u32(floor(constants.width / constants.cell_size));
	return grid_coord.y * cells_per_row + grid_coord.x;
}



@compute @workgroup_size(64)
fn hash_particles(@builtin(global_invocation_id) global_id:vec3<u32>){
    let index = global_id.x;
    if (index >= constants.no_particles) { return; }
    let clamped_pos = clamp(particles[index].pos, vec2<f32>(0.0, 0.0), vec2<f32>(constants.width, constants.height));	
    let grid_coord = hash(grid_coord(clamped_pos));

    cells_ids[index]= grid_coord;
    particle_ids[index]= index;

}

@compute @workgroup_size(64)
fn build_lookups(@builtin(global_invocation_id) global_id:vec3<u32>){
    let index = global_id.x;
    if (index >= constants.no_particles) {return;}
    let cell_id = cells_ids[index];
    
    atomicAdd(&lookups[cell_id].count, 1u);
    
    if index == 0u {
        lookups[cell_id].start_index = index;
    } else{
        let prev_cell_id = cells_ids[index - 1u];
        if cell_id != prev_cell_id{
            lookups[cell_id].start_index = index;
        }
    }
}

