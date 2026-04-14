const PI = 3.141592;
const NEIGHBOUR_CELL_COUNT: u32 = 9;

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

struct Particle {
    pos: vec2<f32>,
    predicted_pos: vec2<f32>,
    vel: vec2<f32>,
    force: vec2<f32>,
    density: f32,
    pressure: f32,
}
struct Cell {
    cell_id: u32,
    particle_id: u32,
}

struct Lookup {
	start_index: u32,	
	count: atomic<u32>
}

fn pcg_hash(seed: u32) -> u32 {
    var state = seed * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn random_f32() -> f32 {
    rand_state = pcg_hash(rand_state);
    return f32(rand_state) / 4294967295.0;
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

var<private> rand_state: u32;

fn spiky_kernel_gradient(pos:vec2<f32>, pos_other:vec2<f32>) -> vec2<f32>{
    // Used for pressure force calculations
    // (-45/(pi*h⁶)) * (h-r)² * r̂ if 0<=r<=h
    // 0 if h<r
    
    let delta = pos - pos_other;
    let r = dot(delta,delta); // magnitude of the vector pointing at particle i
    let norm_coeff = -10.0 / (PI * pow(constants.influence_radius,5)); // -45.0 / (PI * INFLUENCE_RADIUS.powi(6)) for 3D

    if r < 0.00001 * 0.00001 {
        // particles can be in the same position in which case send them in random direction
        let theta = random_f32() * 2.0 * PI;

        let random_dir = vec2(cos(theta), sin(theta));

        return norm_coeff * pow(constants.influence_radius,2) * random_dir;
    }
    if r <= constants.influence_radius * constants.influence_radius {
        let r_sqrt = sqrt(r);
        let r_hat = delta / r_sqrt;
        return norm_coeff * (pow(constants.influence_radius - r_sqrt,2)) * r_hat;
    } else {
        return vec2(0.0, 0.0);
    
    }
}

fn poly_kernel(pos: vec2<f32>, pos_other: vec2<f32>) -> f32 {
    // used for density
    //(315 / (64πh⁹)) * (h² - r²)³  if r <= h
    // 0 if r>h
    let delta = pos - pos_other;
    let r = dot(delta,delta); // magnitude of the vector pointing at particle i
    let norm_coeff = 4.0 / (PI * pow(constants.influence_radius,8)); // 315.0 / (64.0 * PI * INFLUENCE_RADIUS.powi(9)) for 3D
    if r <= constants.influence_radius * constants.influence_radius {
        return norm_coeff * pow((pow(constants.influence_radius,2) - r),3);
    } else {
        return 0.0;
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
	if (index >= constants.no_particles) {
		return;
	}

    integrate(index);
}

fn neighbours() -> array<vec2<i32>, NEIGHBOUR_CELL_COUNT> {
	return array<vec2<i32>, NEIGHBOUR_CELL_COUNT>(
		vec2<i32>(-1, -1), vec2<i32>(0, -1), vec2<i32>(1, -1),
		vec2<i32>(-1, 0), vec2<i32>(0, 0), vec2<i32>(1, 0),
		vec2<i32>(-1, 1), vec2<i32>(0, 1), vec2<i32>(1, 1)
	);
}

fn grid_coord(pos:vec2<f32>) -> vec2<u32>{
    return vec2(
        u32(floor(pos.x / constants.cell_size)),
        u32(floor(pos.y / constants.cell_size))
    );

}

fn hash(grid_coord: vec2<u32>) -> u32 {
    let cells_per_row = u32(floor(constants.width / constants.cell_size));

    return grid_coord.y * cells_per_row + grid_coord.x;
}

fn calculate_density(pos:vec2<f32>,pos_other:vec2<f32>) -> f32{
    return constants.mass * poly_kernel(pos,pos_other);
}

fn calculate_pressure(density:f32) -> f32{
    return constants.gas_constant * (density - constants.rest_density); 
}

@compute @workgroup_size(64)
fn calculate_pressure_density(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= constants.no_particles){
        return;
    }
    particles[index].density = 0.0;
    let grid_width = floor(constants.width / constants.cell_size);
    let grid_height = floor(constants.height / constants.cell_size);

    let grid_coord = grid_coord(particles[index].predicted_pos);
    let grid_neighbours= neighbours();
    for (var i: u32 = 0u; i < NEIGHBOUR_CELL_COUNT; i += 1u) {
        let offset = grid_neighbours[i];
        let neighbour_x = i32(grid_coord.x) + offset.x;
        let neighbour_y = i32(grid_coord.y) + offset.y;
        if neighbour_x >= 0 
            && neighbour_x < i32(grid_width) 
            && neighbour_y >= 0 
            && neighbour_y < i32(grid_height)
        {
            let cell_key = hash(vec2(u32(neighbour_x),u32(neighbour_y)));
            let start_index = lookups[cell_key].start_index;
            let count = atomicLoad(&lookups[cell_key].count);
            for (var j:u32 = 0u ; j<count; j+= 1u){
                let particle_idx = particle_ids[start_index + j];
                particles[index].density += calculate_density(
                    particles[index].predicted_pos,
                    particles[particle_idx].predicted_pos);
            }

        }
    }
    particles[index].pressure = calculate_pressure(particles[index].density);
}
fn calculate_pressure_vector(
    pos:vec2<f32>,
    pos_other:vec2<f32>,
    pressure:f32,
    pressure_other:f32,
    density_other:f32,
) -> vec2<f32> {

    let grad_spiky = spiky_kernel_gradient(pos,pos_other);
    return constants.mass * ((pressure + pressure_other)/ (2.0 * density_other)) * grad_spiky;

}

@compute @workgroup_size(64)
fn calculate_pressure_force(@builtin(global_invocation_id) global_id: vec3<u32>){
    let index = global_id.x;
    if (index >= constants.no_particles){
        return;
    }
    particles[index].force = vec2<f32>(0.0, 0.0);
    rand_state = pcg_hash(index);
    let grid_width = floor(constants.width / constants.cell_size);
    let grid_height = floor(constants.height / constants.cell_size);

    let grid_coord = grid_coord(particles[index].predicted_pos);
    let grid_neighbours= neighbours();
    for (var i: u32 = 0u; i < NEIGHBOUR_CELL_COUNT; i += 1u) {
        let offset = grid_neighbours[i];
        let neighbour_x = i32(grid_coord.x) + offset.x;
        let neighbour_y = i32(grid_coord.y) + offset.y;
        if neighbour_x >= 0 
            && neighbour_x < i32(grid_width) 
            && neighbour_y >= 0 
            && neighbour_y < i32(grid_height)
        {
            let cell_key = hash(vec2(u32(neighbour_x),u32(neighbour_y)));
            let start_index = lookups[cell_key].start_index;
            let count = atomicLoad(&lookups[cell_key].count);
            for (var j:u32 = 0u; j<count; j+= 1u){
                let particle_idx = particle_ids[start_index + j];
                if index == particle_idx{
                    continue;
                }
                particles[index].force -= calculate_pressure_vector(
                    particles[index].predicted_pos,
                    particles[particle_idx].predicted_pos,
                    particles[index].pressure,
                    particles[particle_idx].pressure,
                    particles[particle_idx].density
                );
            }
        }
    }
    particles[index].force += particles[index].density * constants.gravity;
}

fn integrate(index: u32) {	
	let acceleration = particles[index].force / particles[index].density;
	let velocity_old= particles[index].vel; 

	particles[index].vel += acceleration * constants.dt;
	let velocity_length = length(particles[index].vel);

	// Limit velocity so they don't blow up when coming too close to eachother. 
	// Maybe we need a better solution for this.
	if  velocity_length * velocity_length > constants.max_vel * constants.max_vel {
		particles[index].vel = (particles[index].vel / velocity_length) * constants.max_vel;
	}

	particles[index].pos += (particles[index].vel + velocity_old) * 0.5 * constants.dt;
    boundaries(index);
    particles[index].predicted_pos = particles[index].pos + particles[index].vel * constants.dt;
}

fn boundaries(index: u32) {
    if constants.width - constants.radius < particles[index].pos.x {
        particles[index].pos.x = constants.width - constants.radius;
        particles[index].vel.x *= -constants.damping;
    }		

    if constants.height - constants.radius < particles[index].pos.y {
        particles[index].pos.y = constants.height - constants.radius;
        particles[index].vel.y *= -constants.damping;
    }

    if particles[index].pos.x < constants.radius {
        particles[index].pos.x = constants.radius;
        particles[index].vel.x *= -constants.damping;
    }

    if particles[index].pos.y < constants.radius {
        particles[index].pos.y = constants.radius;
        particles[index].vel.y *= -constants.damping;
    }
}
