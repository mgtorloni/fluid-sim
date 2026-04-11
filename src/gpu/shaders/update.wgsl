const PI = 3.141592;
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

fn random_f32(seed: u32) -> f32 {
    return f32(pcg_hash(seed)) / 4294967295.0;
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

fn spiky_kernel_gradient(pos:vec2<f32>, pos_other:vec2<f32>, constants:Constants, random_val:f32) -> vec2<f32>{
    // Used for pressure force calculations
    // (-45/(pi*h⁶)) * (h-r)² * r̂ if 0<=r<=h
    // 0 if h<r
    let delta = pos - pos_other;
    let r = dot(delta,delta); // magnitude of the vector pointing at particle i
    let norm_coeff = -10.0 / (PI * pow(constants.influence_radius,5)); // -45.0 / (PI * INFLUENCE_RADIUS.powi(6)) for 3D

    if r < 0.00001 * 0.0001 {
        // particles can be in the same position in which case send them in random direction
        let theta = random_val * 2.0 * PI;

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

fn poly_kernel(pos: vec2<f32>, pos_other: vec2<f32>, constants: Constants) -> f32 {
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

	update(index);
    integrate(index);
}

fn find_cell_start(index: u32) {
	let cell_id = cells_ids[index];

    if index == 0u {
        lookups[cell_id].start_index = index;
    } else {
        let prev_cell_id = cells_ids[index - 1u];

        if cell_id != prev_cell_id {
            lookups[cell_id].start_index = index;
        }
    }
    atomicAdd(&lookups[cell_id].count, 1u);
	
}


fn update(index: u32) {


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

	particles[index].vel += constants.gravity * constants.dt;
	particles[index].pos += particles[index].vel * constants.dt;
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
