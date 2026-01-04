use crate::vec2;
use macroquad::prelude::Vec2;

//FIX: THESE CONSTANTS MAKE NO SENSE,GRAVITY IS STRANGE
pub const NO_PARTICLES: usize = 600;
pub const RADIUS: f32 = 6.0;
pub const GRAVITY: Vec2 = vec2(0.0, 0.0);
pub const DAMPING: f32 = 0.5;
pub const WALL_PRESSURE_FORCE: f32 = 200.0;
pub const INFLUENCE_RADIUS: f32 = 50.0;
pub const MASS: f32 = 1.0;
pub const GAS_CONSTANT: f32 = 461.5; // 461.5 J/(kg*K) for water
pub const REST_DENSITY: f32 = 1000.0; // 1000kg/m^3 for water
