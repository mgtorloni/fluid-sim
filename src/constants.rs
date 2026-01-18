use crate::glam::Vec2;
use crate::vec2;

pub const NO_PARTICLES: usize = 800;
pub const MAX_VEL: f32 = 7000.0;
// pub const SCALE: f32 = 50.0;
pub const RADIUS: f32 = 5.0;
pub const MASS: f32 = 1.0;
pub const REST_DENSITY: f32 = 0.0014;
// pub const GRAVITY: Vec2 = vec2(0.0, 0.0);
pub const GRAVITY: Vec2 = vec2(0.0, 80000.0);
pub const GAS_CONSTANT: f32 = 100000000.0;
pub const INFLUENCE_RADIUS: f32 = 30.0;
pub const DAMPING: f32 = 0.8;
pub const MOUSE_INFLUENCE_RADIUS: f32 = 70.0;
pub const MOUSE_FORCE_STRENGTH: f32 = 10000.0;
