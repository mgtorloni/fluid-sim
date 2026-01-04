use crate::Vec2;
use crate::constants::{GAS_CONSTANT, GRAVITY, MASS, REST_DENSITY};
use crate::kernels::{poly_kernel, spiky_kernel_gradient};

pub fn calculate_pressure(density: f32) -> f32 {
    GAS_CONSTANT * (density - REST_DENSITY)
}

pub fn calculate_pressure_force(
    pos: Vec2,
    pos_other: Vec2,
    pressure: f32,
    pressure_other: f32,
    density_other: f32,
) -> Vec2 {
    let grad_spiky = spiky_kernel_gradient(pos, pos_other);

    MASS * ((pressure + pressure_other) / (2.0 * density_other)) * grad_spiky
}

pub fn calculate_gravity_force(density: f32) -> Vec2 {
    density * GRAVITY
}
pub fn calculate_density(pos: Vec2, pos_other: Vec2) -> f32 {
    MASS * poly_kernel(pos, pos_other)
}
