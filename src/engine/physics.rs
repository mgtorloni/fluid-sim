use super::kernels::{poly_kernel, spiky_kernel_gradient};
use crate::Vec2;
use crate::constants::{GAS_CONSTANT, GRAVITY, MASS, REST_DENSITY};

pub fn calculate_pressure(density: f32) -> f32 {
    let pressure = GAS_CONSTANT * (density - REST_DENSITY);
    pressure
    // pressure.max(0.0)
    // let gamma = 7.0;
    // let B = GAS_CONSTANT * REST_DENSITY / gamma;
    //
    // // Tait equation: Non-linear response (stiff)
    // B * ((density / REST_DENSITY).powf(gamma) - 1.0)
}

pub fn calculate_pressure_force(
    pos: Vec2,
    pos_other: Vec2,
    pressure: f32,
    pressure_other: f32,
    // density: f32,
    density_other: f32,
) -> Vec2 {
    let grad_spiky = spiky_kernel_gradient(pos, pos_other);

    MASS * ((pressure + pressure_other) / (2.0 * density_other)) * grad_spiky
    // MASS * ((pressure / density.powi(2)) + (pressure_other / density_other.powi(2))) * grad_spiky
}

pub fn calculate_gravity_force(density: f32) -> Vec2 {
    density * GRAVITY
}
pub fn calculate_density(pos: Vec2, pos_other: Vec2) -> f32 {
    MASS * poly_kernel(pos, pos_other)
}
