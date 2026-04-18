struct Constants {
	width:f32,
	height:f32,
	no_particles:u32,
	max_vel: f32,
    radius: f32,
    mass: f32,
    rest_density: f32,
	dt: f32,
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

@group(0) @binding(0) 
var<storage, read_write> particles: array<Particle>;

@group(0) @binding(1) 
var<uniform> constants: Constants;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>, 
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput {
    let particle = particles[instance_index];

    // Hardcode a square
    var pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>( 1.0, -1.0), vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0), vec2<f32>( 1.0,  1.0), vec2<f32>(-1.0,  1.0)
    );
    let quad_pos = pos[vertex_index];
    

	let pixel_pos = particle.pos + (quad_pos * constants.radius);
	let screen_size = vec2<f32>(constants.width, constants.height);
    let clip_pos = (pixel_pos / screen_size) * 2.0 - 1.0;    

    let final_clip_pos = vec2<f32>(clip_pos.x, -clip_pos.y);

    var out: VertexOutput;
    out.clip_position = vec4<f32>(final_clip_pos, 0.0, 1.0);
    out.local_pos = quad_pos; // Pass local -1 to +1 coordinate to fragment shader
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Shave off the corners to make a perfect circle
    if dot(in.local_pos, in.local_pos) > 1.0 {
        discard; 
    }
    
    // Blue!
    return vec4<f32>(0.1, 0.5, 1.0, 1.0);
}
