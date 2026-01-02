use crate::vec2;
use macroquad::prelude::Vec2;

//FIX: THESE CONSTANTS MAKE NO SENSE
pub const RADIUS: f32 = 6.0;
pub const GRAVITY: Vec2 = vec2(0.0, 0.0);
pub const DAMPING: f32 = 0.5;
pub const WALL_PRESSURE_FORCE: f32 = 200.0;
pub const INFLUENCE_RADIUS: f32 = 50.0;
pub const MASS: f32 = 1.0;
pub const GAS_CONSTANT: f32 = 100000.0;
pub const REST_DENSITY: f32 = 1.0;
