use crate::glam::Vec2;
use crate::vec2;

pub const NO_PARTICLES: usize = 2300;
pub const MAX_VEL: f32 = 5000.0;
pub const WIDTH: i32 = 1000;
pub const HEIGHT: i32 = 1000;
// pub const SCALE: f32 = 50.0;
pub const RADIUS: f32 = 5.0;
pub const MASS: f32 = 1.0;
pub const REST_DENSITY: f32 = 0.0014;
// pub const GRAVITY: Vec2 = vec2(0.0, 0.0);
pub const GRAVITY: Vec2 = vec2(0.0, 80000.0);
pub const GAS_CONSTANT: f32 = 50000000.0;
pub const INFLUENCE_RADIUS: f32 = 40.0;
pub const CELL_SIZE: f32 = INFLUENCE_RADIUS;
pub const DAMPING: f32 = 0.8;
pub const MOUSE_INFLUENCE_RADIUS: f32 = 70.0;
pub const MOUSE_FORCE_STRENGTH: f32 = 1000.0;
