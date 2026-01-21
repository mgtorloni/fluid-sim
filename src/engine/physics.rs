use super::kernels::{poly_kernel, spiky_kernel_gradient};
use crate::Vec2;
use crate::constants::SimulationParams;

pub fn calculate_pressure(density: f32, params: &SimulationParams) -> f32 {
    let pressure = params.gas_constant * (density - params.rest_density);
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
    params: &SimulationParams,
) -> Vec2 {
    let grad_spiky = spiky_kernel_gradient(pos, pos_other, &params);

    params.mass * ((pressure + pressure_other) / (2.0 * density_other)) * grad_spiky
    // MASS * ((pressure / density.powi(2)) + (pressure_other / density_other.powi(2))) * grad_spiky
}

pub fn calculate_gravity_force(density: f32, params: &SimulationParams) -> Vec2 {
    density * params.gravity
}
pub fn calculate_density(pos: Vec2, pos_other: Vec2, params: &SimulationParams) -> f32 {
    params.mass * poly_kernel(pos, pos_other, &params)
}
