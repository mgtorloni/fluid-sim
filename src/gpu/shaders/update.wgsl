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


@group(0) @binding(0) 
var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) 
var<uniform> constants: Constants;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
	if (index >= constants.no_particles) {
		return;
	}

	update(index);
    integrate(index);
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
