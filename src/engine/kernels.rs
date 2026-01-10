use crate::constants::INFLUENCE_RADIUS;
use crate::rand;
use crate::vec2;
use macroquad::prelude::Vec2;
use std::f32::consts::PI;

pub fn spiky_kernel_gradient(pos: Vec2, pos_other: Vec2) -> Vec2 {
    // Used for pressure force calculations
    // (-45/(pi*h⁶)) * (h-r)² * r̂ if 0<=r<=h
    // 0 if h<r
    let delta = pos - pos_other;
    let r = delta.length(); // magnitude of the vector pointing at particle j
    let norm_coeff = -10.0 / (PI * INFLUENCE_RADIUS.powf(5.0)); // -45.0 / (PI * INFLUENCE_RADIUS.powf(6.0)) for 3D

    if r < 0.00001 {
        // particles can be in the same position in which case send them in random direction
        let theta = rand::gen_range(0.0, 2.0 * PI);

        let random_dir = vec2(theta.cos(), theta.sin());

        return norm_coeff * (INFLUENCE_RADIUS).powf(2.0) * random_dir;
    }
    let r_hat = delta / r;
    if r <= INFLUENCE_RADIUS {
        norm_coeff * (INFLUENCE_RADIUS - r).powf(2.0) * r_hat
    } else {
        vec2(0.0, 0.0)
    }
}

pub fn poly_kernel(pos: Vec2, pos_other: Vec2) -> f32 {
    // used for density
    //(315 / (64πh⁹)) * (h² - r²)³  if r <= h
    // 0 if r>h
    let delta = pos - pos_other;
    let r = delta.length(); // magnitude of the vector pointing at particle j
    let norm_coeff = 4.0 / (PI * INFLUENCE_RADIUS.powf(8.0)); // 315.0 / (64.0 * PI * INFLUENCE_RADIUS.powf(9.0)) for 3D
    if r <= INFLUENCE_RADIUS {
        norm_coeff * (INFLUENCE_RADIUS.powf(2.0) - r.powf(2.0)).powf(3.0)
    } else {
        0.0
    }
}
