use crate::constants::*;
use crate::engine::physics::{
    calculate_density, calculate_gravity_force, calculate_pressure, calculate_pressure_force,
};
use crate::glam::Vec2;
use rayon::prelude::*;

pub type ParticleVector = Vec2;
pub type ParticleScalar = f32;

#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: ParticleVector,
    pub vel: ParticleVector,
    pub density: ParticleScalar,
    pub pressure: ParticleScalar,
    pub force: ParticleVector,
}

pub struct Particles {
    pub pos: Vec<ParticleVector>,
    pub vel: Vec<ParticleVector>,
    pub density: Vec<ParticleScalar>,
    pub pressure: Vec<ParticleScalar>,
    pub force: Vec<ParticleVector>,
}

impl Particles {
    pub fn new() -> Self {
        Self {
            pos: Vec::new(),
            vel: Vec::new(),
            density: Vec::new(),
            pressure: Vec::new(),
            force: Vec::new(),
        }
    }

    pub fn spawn(&mut self, particle: Particle) {
        self.pos.push(particle.pos);
        self.vel.push(particle.vel);
        self.density.push(particle.density);
        self.pressure.push(particle.pressure);
        self.force.push(particle.force);
    }

    pub fn boundaries(world_size: Vec2, pos: &mut Vec2, vel: &mut Vec2) {
        let world_width = world_size.x;
        let world_height = world_size.y;
        // let world_width = screen_width() / SCALE;
        // let world_height = screen_height() / SCALE;

        let particle_radius_m = RADIUS / SCALE;

        if pos.x >= world_width - particle_radius_m {
            // if vel.y <= 0.5 {
            //     vel.x = 0.0;
            // }
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

    fn mouse_action(pos: Vec2, mouse_pos: Vec2, interaction_strength: f32) -> Vec2 {
        if interaction_strength != 0.0 {
            let delta = {
                if interaction_strength > 0.0 {
                    pos - mouse_pos
                } else {
                    mouse_pos - pos
                }
            };
            let dist = delta.length();

            if dist < MOUSE_INFLUENCE_RADIUS && dist > 0.0001 {
                let dir = delta / dist;
                let factor = if interaction_strength > 0.0 {
                    // stronger in the center 0 at the edge
                    1.0 - dist / MOUSE_INFLUENCE_RADIUS
                } else {
                    // stronger in the ed
                    dist / MOUSE_INFLUENCE_RADIUS
                };

                return dir * interaction_strength.abs() * factor;
            } else {
                return Vec2::ZERO;
            }
        }
        Vec2::ZERO
    }

    pub fn integrate(
        &mut self,
        world_size: Vec2,
        mouse_pos: Vec2,
        interaction_strength: f32,
        dt: f32,
    ) {
        for i in 0..NO_PARTICLES {
            let acceleration = self.force[i] / self.density[i];

            self.vel[i] += acceleration * dt;

            let mouse_vel = Self::mouse_action(self.pos[i], mouse_pos, interaction_strength);
            self.vel[i] += mouse_vel;

            if self.vel[i].length_squared() > MAX_VEL * MAX_VEL {
                self.vel[i] = (self.vel[i] / self.vel[i].length()) * MAX_VEL;
            }

            self.pos[i] += self.vel[i] * dt;

            Self::boundaries(world_size, &mut self.pos[i], &mut self.vel[i]);
        }
    }

    pub fn update(&mut self) {
        let positions = &self.pos;
        self.density
            .par_iter_mut()
            .enumerate()
            .zip(self.pressure.par_iter_mut())
            .for_each(|((i, density_ref), pressure_ref)| {
                let mut current_density: f32 = 0.0;

                for j in 0..NO_PARTICLES {
                    current_density += calculate_density(self.pos[i], self.pos[j]);
                }
                *density_ref = current_density;
                *pressure_ref = calculate_pressure(*density_ref);
            });
        let pressures = &self.pressure;
        let densities = &self.density;

        self.force
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, force_ref)| {
                let mut current_force = Vec2::ZERO;

                for j in 0..NO_PARTICLES {
                    if i == j {
                        continue;
                    }

                    let pressure_force = calculate_pressure_force(
                        positions[i],
                        positions[j],
                        pressures[i],
                        pressures[j],
                        densities[j],
                    );

                    current_force -= pressure_force;
                }

                let gravity_force = calculate_gravity_force(densities[i]);
                current_force += gravity_force;

                *force_ref = current_force;
            });
    }
}
