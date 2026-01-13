use crate::constants::*;
use crate::engine::simulation::Particles;
use colorgrad::{Gradient, GradientBuilder, LinearGradient};
use macroquad::prelude::*;

pub struct FluidRenderer {
    gradient: LinearGradient,
}

impl FluidRenderer {
    pub fn new() -> Self {
        let gradient = GradientBuilder::new()
            .colors(&[
                colorgrad::Color::from_rgba8(50, 100, 255, 255),
                colorgrad::Color::from_rgba8(0, 200, 120, 255),
                colorgrad::Color::from_rgba8(240, 230, 50, 255),
                colorgrad::Color::from_rgba8(255, 160, 60, 255),
                colorgrad::Color::from_rgba8(255, 0, 0, 255),
            ])
            .domain(&[0.0, 0.25, 0.5, 0.75, 0.90])
            .build::<LinearGradient>()
            .expect("Failed to build gradient");
        Self { gradient }
    }
    pub fn draw(&self, particles: &Particles) {
        for i in 0..particles.pos.len() {
            let pixel_pos = particles.pos[i] * SCALE;

            let speed = particles.vel[i].length();
            let t = speed / MAX_VEL;

            let [r, g, b, a] = self.gradient.at(t).to_rgba8();
            let color = Color::from_rgba(r, g, b, a);

            draw_circle(pixel_pos.x, pixel_pos.y, RADIUS, color);
        }
    }
}
