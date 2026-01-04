use colorgrad::{Gradient, GradientBuilder, LinearGradient};
use macroquad::prelude::*;
mod constants;
mod kernels;
mod physics;
use crate::constants::*;
use crate::physics::{
    calculate_density, calculate_gravity_force, calculate_pressure, calculate_pressure_force,
};

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

    fn boundaries(&mut self, dt: f32, i: usize) {
        self.vel[i] += GRAVITY * dt; // v = u + at
        self.pos[i] += self.vel[i] * dt; // s = u + vt
        if self.pos[i].x >= screen_width() - RADIUS {
            self.vel[i].x = -self.vel[i].x * DAMPING - (WALL_PRESSURE_FORCE) * dt;
            self.pos[i].x = screen_width() - RADIUS;
        } else if self.pos[i].x <= RADIUS {
            self.vel[i].x = -self.vel[i].x * DAMPING + (WALL_PRESSURE_FORCE) * dt;
            self.pos[i].x = RADIUS;
        }
        if self.pos[i].y >= screen_height() - RADIUS {
            self.vel[i].y = -self.vel[i].y * DAMPING - (WALL_PRESSURE_FORCE) * dt;
            self.pos[i].y = screen_height() - RADIUS;
        } else if self.pos[i].y <= RADIUS {
            self.vel[i].y = -self.vel[i].y * DAMPING + (WALL_PRESSURE_FORCE) * dt;
            self.pos[i].y = RADIUS;
        }
    }
    fn update(&mut self, dt: f32) {
        for i in 0..NO_PARTICLES {
            self.boundaries(dt, i);
            self.density[i] = 0.0;
            self.pressure[i] = 0.0;
            self.force[i] = vec2(0.0, 0.0);
            for j in 0..NO_PARTICLES {
                self.density[i] += calculate_density(self.pos[i], self.pos[j]);
            }
            self.pressure[i] += calculate_pressure(self.density[i]);
        }
        for i in 0..NO_PARTICLES {
            println!("Pressure:{}", self.pressure[i]);
            println!("Density: {}", self.density[i]);
            //FIX: DENSITY VALUES ARE SO SMALL THAT PRESSURE IS ALWAYS THE SAME
            for j in 0..NO_PARTICLES {
                if i == j {
                    //NOTE: if i=j then the distance between the "two" particles
                    // is 0 and that grad_spiky will be NaN causing pressure force calculation to fail
                    continue;
                }
                let pressure_force = calculate_pressure_force(
                    self.pos[i],
                    self.pos[j],
                    self.pressure[i],
                    self.pressure[j],
                    self.density[j],
                );
                let gravity_force = calculate_gravity_force(self.density[i]);
                self.force[i] -= pressure_force;
                self.force[i] += gravity_force;
            }
            self.vel[i] += (self.force[i] / MASS) * dt;
        }
    }

    fn draw(&self) {
        let gradient = GradientBuilder::new()
            .colors(&[
                colorgrad::Color::from_rgba8(255, 255, 255, 255), // blue
                colorgrad::Color::from_rgba8(255, 0, 0, 255),     // red
            ])
            .build::<LinearGradient>()
            .expect("Failed to build gradient");

        for i in 0..self.pos.len() {
            let pos = self.pos[i];
            let speed = self.vel[i].length();

            // Normalize speed into [0, 1]
            let t = (speed / 1000.0).clamp(0.0, 1.0);

            let [r, g, b, a] = gradient.at(t).to_rgba8();
            let color = Color::from_rgba(r, g, b, a);

            draw_circle(pos.x, pos.y, RADIUS, color);
        }
    }
}

fn conf() -> Conf {
    Conf {
        window_title: "fluidsim".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut simulation = Particles::new();
    next_frame().await;
    for _ in 0..NO_PARTICLES {
        simulation.spawn(Particle {
            pos: vec2(
                rand::gen_range(0.0, screen_width()),
                rand::gen_range(0.0, screen_height()),
            ),
            vel: vec2(0.0, 0.0),
            density: 0.0,
            pressure: 0.0,
            force: vec2(0.0, 0.0),
            // rand::gen_range(-50.0, 50.0),
        });
    }

    loop {
        let dt = 1.0 / 60.0;
        clear_background(DARKGRAY);
        simulation.update(dt);
        simulation.draw();
        next_frame().await;
    }
}
