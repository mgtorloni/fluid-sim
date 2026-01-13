use crate::glam::Vec2;
use crate::vec2;

pub const NO_PARTICLES: usize = 800;
pub const MAX_VEL: f32 = 90.0;
pub const SCALE: f32 = 50.0;
pub const RADIUS: f32 = 6.0;
pub const MASS: f32 = 100.0;
pub const REST_DENSITY: f32 = 1.0;
pub const GRAVITY: Vec2 = vec2(0.0, 0.0);
pub const GAS_CONSTANT: f32 = 5000.0;
pub const INFLUENCE_RADIUS: f32 = 0.6;
pub const DAMPING: f32 = 0.8;
pub const MOUSE_INFLUENCE_RADIUS: f32 = 2.0;
pub const MOUSE_FORCE_STRENGTH: f32 = 15.0;
