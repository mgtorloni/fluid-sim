// Shared type definitions. Prepended to each shader module at runtime
// in pipelines.rs. Keep in sync with SimulationParams / GpuParticle in Rust.

struct Constants {
    width: f32,
    height: f32,
    no_particles: u32,
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
    mouse_pos: vec2<f32>,
    mouse_strength: f32,
    mouse_influence_radius: f32,
    _padding: vec2<f32>,
}

struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
    force: vec2<f32>,
    density: f32,
    pressure: f32,
}

struct Lookup {
    start_index: u32,
    end_index: u32,
}
