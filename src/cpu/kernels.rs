use crate::constants::SimulationParams;
use crate::glam::Vec2;
use crate::rand;
use crate::vec2;
use std::f32::consts::PI;

pub fn spiky_kernel_gradient(pos: Vec2, pos_other: Vec2, params: &SimulationParams) -> Vec2 {
    // Used for pressure force calculations
    // (-45/(pi*h⁶)) * (h-r)² * r̂ if 0<=r<=h
    // 0 if h<r
    let delta = pos - pos_other;
    let r = delta.length_squared(); // magnitude of the vector pointing at particle i
    let norm_coeff = -10.0 / (PI * params.influence_radius.powi(5)); // -45.0 / (PI * INFLUENCE_RADIUS.powi(6)) for 3D

    if r < 0.00001 * 0.0001 {
        // particles can be in the same position in which case send them in random direction
        let theta = rand::gen_range(0.0, 2.0 * PI);

        let random_dir = vec2(theta.cos(), theta.sin());

        return norm_coeff * params.influence_radius.powi(2) * random_dir;
    }
    if r <= params.influence_radius * params.influence_radius {
        let r_sqrt = r.sqrt();
        let r_hat = delta / r_sqrt;
        norm_coeff * (params.influence_radius - r_sqrt).powi(2) * r_hat
    } else {
        vec2(0.0, 0.0)
    }
}

pub fn poly_kernel(pos: Vec2, pos_other: Vec2, params: &SimulationParams) -> f32 {
    // used for density
    //(315 / (64πh⁹)) * (h² - r²)³  if r <= h
    // 0 if r>h
    let delta = pos - pos_other;
    let r = delta.length_squared(); // magnitude of the vector pointing at particle i
    let norm_coeff = 4.0 / (PI * params.influence_radius.powi(8)); // 315.0 / (64.0 * PI * INFLUENCE_RADIUS.powi(9)) for 3D
    if r <= params.influence_radius * params.influence_radius {
        norm_coeff * (params.influence_radius.powi(2) - r).powi(3)
    } else {
        0.0
    }
}
