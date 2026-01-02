use macroquad::prelude::*;

mod constants;
use crate::constants::*;

mod kernels;
use crate::kernels::{poly_kernel, spiky_kernel_gradient};

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

    fn boundaries(&mut self, dt: f32) {
        let count = self.pos.len();
        for i in 0..count {
            // self.vel[i].y += GRAVITY * dt; // v = u + at
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
    }

    fn calculate_pressure_force(&mut self, i: usize, j: usize) -> Vec2 {
        let grad_spiky = spiky_kernel_gradient(self.pos[i], self.pos[j]);

        MASS * ((self.pressure[i] + self.pressure[j]) / (2.0 * self.density[j])) * grad_spiky
    }

    fn calculate_gravity_force(&mut self, i: usize) -> Vec2 {
        self.density[i] * GRAVITY
    }

    fn update(&mut self, dt: f32) {
        self.boundaries(dt);
        let count = self.pos.len();
        for i in 0..count {
            self.density[i] = 0.0;
            self.pressure[i] = 0.0;
            self.force[i] = vec2(0.0, 0.0);
            for j in 0..count {
                self.density[i] += MASS * poly_kernel(self.pos[i], self.pos[j]);
            }
            println!("{}", self.density[i]);
            self.pressure[i] += GAS_CONSTANT * (self.density[i] - REST_DENSITY);
        }
        for i in 0..count {
            for j in 0..count {
                if i == j {
                    //NOTE: if i=j then the distance between the "two" particles
                    //is 0 and that grad_spiky will be NaN causing force calculation to fail
                    continue;
                }
                let pressure_force = self.calculate_pressure_force(i, j);
                let gravity_force = self.calculate_gravity_force(i);
                self.force[i] += pressure_force;
                self.force[i] += gravity_force;
            }
            self.vel[i] += (self.force[i] / MASS) * dt;
        }
    }

    fn draw(&self) {
        let max_speed: f32 = 100.0;

        for i in 0..self.pos.len() {
            let ratio: f32 = (self.vel[i].length() / max_speed).min(1.0);
            // let colour = Color::new(
            //     (ratio * 2.0).min(1.0),
            //     1.0 - (ratio - 0.5).abs() * 2.0,
            //     ((1.0 - ratio) * 2.0).min(1.0),
            //     1.0,
            // );
            let colour = Color::new(ratio, 0.0, 1.0 - ratio, 1.0);
            draw_circle(self.pos[i].x, self.pos[i].y, RADIUS, WHITE);
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

    for _ in 0..600 {
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
        let dt = get_frame_time();
        clear_background(DARKGRAY);
        simulation.update(dt);
        simulation.draw();
        next_frame().await;
    }
}
