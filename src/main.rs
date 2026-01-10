mod constants;
mod engine;

use crate::constants::*;
use colorgrad::{Gradient, GradientBuilder, LinearGradient};
use engine::physics::{
    calculate_density, calculate_gravity_force, calculate_pressure, calculate_pressure_force,
};
use macroquad::prelude::*;
use rayon::prelude::*;

type ParticleVector = Vec2;
type ParticleScalar = f32;

struct Particle {
    pos: ParticleVector,
    vel: ParticleVector,
    density: ParticleScalar,
    pressure: ParticleScalar,
    force: ParticleVector,
}

struct Particles {
    pos: Vec<ParticleVector>,
    vel: Vec<ParticleVector>,
    density: Vec<ParticleScalar>,
    pressure: Vec<ParticleScalar>,
    force: Vec<ParticleVector>,
}

impl Particles {
    fn new() -> Self {
        Self {
            pos: Vec::new(),
            vel: Vec::new(),
            density: Vec::new(),
            pressure: Vec::new(),
            force: Vec::new(),
        }
    }

    fn spawn(&mut self, particle: Particle) {
        self.pos.push(particle.pos);
        self.vel.push(particle.vel);
        self.density.push(particle.density);
        self.pressure.push(particle.pressure);
        self.force.push(particle.force);
    }

    fn boundaries(pos: &mut Vec2, vel: &mut Vec2) {
        let world_width = screen_width() / SCALE;
        let world_height = screen_height() / SCALE;
        let particle_radius_m = RADIUS / SCALE;

        if pos.x >= world_width - particle_radius_m {
            vel.x = -vel.x * DAMPING;
            pos.x = world_width - particle_radius_m;
        } else if pos.x <= particle_radius_m {
            vel.x = -vel.x * DAMPING;
            pos.x = particle_radius_m;
        }

        if pos.y >= world_height - particle_radius_m {
            vel.y = -vel.y * DAMPING;
            pos.y = world_height - particle_radius_m;
        } else if pos.y <= particle_radius_m {
            vel.y = -vel.y * DAMPING;
            pos.y = particle_radius_m;
        }
    }

    fn mouse_action(pos: &Vec2) -> Vec2 {
        let (mx, my) = mouse_position();
        let mouse_pos = vec2(mx / SCALE, my / SCALE);
        let is_pushing = is_mouse_button_down(MouseButton::Left);

        if is_pushing {
            let delta = *pos - mouse_pos;
            let dist = delta.length();

            if dist < MOUSE_INFLUENCE_RADIUS && dist > 0.0001 {
                let dir = delta / dist;
                // stronger in the center 0 at the edge
                let strength = MOUSE_FORCE_STRENGTH * (1.0 - dist / MOUSE_INFLUENCE_RADIUS);
                return dir * strength;
            } else {
                return Vec2::ZERO;
            }
        }
        Vec2::ZERO
    }

    fn update(&mut self, dt: f32) {
        for i in 0..NO_PARTICLES {
            self.density[i] = 0.0;
            self.pressure[i] = 0.0;
            self.force[i] = Vec2::ZERO;

            for j in 0..NO_PARTICLES {
                self.density[i] += calculate_density(self.pos[i], self.pos[j]);
            }

            self.pressure[i] = calculate_pressure(self.density[i]);
        }
        let pos = &mut self.pos;
        let pressure = &self.pressure;
        let density = &self.density;
        let force = &mut self.force; // We need to write to this
        let vel = &mut self.vel;
        (0..NO_PARTICLES).for_each(|i| {
            (0..NO_PARTICLES).for_each(|j| {
                if i == j {
                    return;
                }

                let pressure_force =
                    calculate_pressure_force(pos[i], pos[j], pressure[i], pressure[j], density[j]);

                force[i] -= pressure_force;
            });

            let gravity_force = calculate_gravity_force(density[i]);
            force[i] += gravity_force;

            let acceleration = force[i] / density[i];

            vel[i] += acceleration * dt;
            let mouse_vel = Self::mouse_action(&pos[i]);
            vel[i] += mouse_vel;

            pos[i] += vel[i] * dt;
            Self::boundaries(&mut pos[i], &mut vel[i]);
        });
    }

    fn draw(&self) {
        let gradient = GradientBuilder::new()
            .colors(&[
                colorgrad::Color::from_rgba8(50, 100, 255, 255),
                colorgrad::Color::from_rgba8(255, 150, 150, 255),
            ])
            .build::<LinearGradient>()
            .expect("Failed to build gradient");

        for i in 0..self.pos.len() {
            let pixel_pos = self.pos[i] * SCALE;

            let speed = self.vel[i].length();
            let t = (speed / 5.0).clamp(0.0, 1.0);

            let [r, g, b, a] = gradient.at(t).to_rgba8();
            let color = Color::from_rgba(r, g, b, a);

            draw_circle(pixel_pos.x, pixel_pos.y, RADIUS, color);
        }
    }
}

fn conf() -> Conf {
    Conf {
        window_title: "fluidsim".to_owned(),
        window_height: 600,
        window_width: 600,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut simulation = Particles::new();

    // let cols = 20;
    // let rows = 30;
    // let spacing = 0.1; // 10cm spacing
    //
    // let grid_width = cols as f32 * spacing;
    // let grid_height = rows as f32 * spacing;
    //
    let world_width = screen_width() / SCALE;
    let world_height = screen_height() / SCALE;
    //
    // let offset_x = (world_width - grid_width) / 2.0;
    // let offset_y = (world_height - grid_height) / 2.0;
    //
    // for y in 0..rows {
    //     for x in 0..cols {
    //         simulation.spawn(Particle {
    //             pos: vec2(offset_x + x as f32 * spacing, offset_y + y as f32 * spacing),
    //             vel: vec2(0.0, 0.0),
    //             density: REST_DENSITY,
    //             pressure: 0.0,
    //             force: vec2(0.0, 0.0),
    //         });
    //     }
    // }

    for _ in 0..NO_PARTICLES {
        simulation.spawn(Particle {
            pos: vec2(
                rand::gen_range(0.0, world_width),
                rand::gen_range(0.0, world_height),
            ),
            vel: vec2(0.0, 0.0),
            density: REST_DENSITY,
            pressure: 0.0,
            force: vec2(0.0, 0.0),
        });
    }
    loop {
        let dt = 0.002;

        clear_background(BLACK);

        simulation.update(dt);
        simulation.draw();

        draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 30.0, WHITE);

        next_frame().await;
    }
}
