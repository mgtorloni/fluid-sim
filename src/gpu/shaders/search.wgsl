
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

@group(0) @binding(1) 
var<uniform> constants: Constants;
@group(0) @binding(2) 
var<storage, read_write> cells: array<Cell>;
@group(0) @binding(3)
var<storage, read_write> lookups: array<Lookup>;

fn grid_coord(pos: vec2<f32>) -> vec2<u32> {
	return vec2<u32>(floor(pos / constants.cell_size));
}

fn hash(pos: vec2<f32>, grid_coord:vec2<u32>) -> u32 {
	let cells_per_row: u32 = u32(floor(constants.width / constants.cell_size));
	return grid_coord.y * cells_per_row + grid_coord.x;
}

fn neighbours() -> array<vec2<i32>, 9> {
	return array<vec2<i32>, 9>(
		vec2<i32>(-1, -1), vec2<i32>(0, -1), vec2<i32>(1, -1),
		vec2<i32>(-1, 0), vec2<i32>(0, 0), vec2<i32>(1, 0),
		vec2<i32>(-1, 1), vec2<i32>(0, 1), vec2<i32>(1, 1)
	);
}

fn find_cell_start(index: u32) {
	let cell_id = cells[index].cell_id;

    if index == 0u {
        lookups[cell_id].start_index = index;
    } else {
        let prev_cell_id = cells[index - 1u].cell_id;

        if cell_id != prev_cell_id {
            lookups[cell_id].start_index = index;
        }
    }
    atomicAdd(&lookups[cell_id].count, 1u);
	
}

//let clamped_pos = clamp(particles[index].pos, vec2<f32>(0.0, 0.0), vec2<f32>(constants.width, constants.height));	
//let grid_coord = hash(clamped_pos, grid_coord(clamped_pos));

//cells[index].cell_id = grid_coord;
//cells[index].particle_id = index;

//find_cell_start(index);
//sort(cells);
