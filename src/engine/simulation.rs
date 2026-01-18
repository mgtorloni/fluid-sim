use crate::constants::*;
use crate::engine::physics::{
    calculate_density, calculate_gravity_force, calculate_pressure, calculate_pressure_force,
};
use crate::engine::search;
use crate::glam::{IVec2, Vec2};
use crate::uvec2;
use rayon::prelude::*;
use rdxsort::*;

pub type ParticleVector = Vec2;
pub type ParticleScalar = f32;

#[derive(Clone, Copy)]
pub enum IOInteraction {
    None,
    Repel(f32),
    Attract(f32),
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: ParticleVector,
    pub predicted_pos: ParticleVector,
    pub vel: ParticleVector,
    pub density: ParticleScalar,
    pub pressure: ParticleScalar,
    pub force: ParticleVector,
}

pub struct Particles {
    pub pos: Vec<ParticleVector>,
    pub predicted_pos: Vec<ParticleVector>,
    pub vel: Vec<ParticleVector>,
    pub density: Vec<ParticleScalar>,
    pub pressure: Vec<ParticleScalar>,
    pub force: Vec<ParticleVector>,
}

impl IOInteraction {
    pub fn delta_vel(&self, particle_pos: Vec2, io_pos: Vec2) -> Vec2 {
        match self {
            IOInteraction::None => Vec2::ZERO,
            IOInteraction::Repel(strength) => {
                let delta = particle_pos - io_pos; // points away from IO
                let dist = delta.length();

                if dist < MOUSE_INFLUENCE_RADIUS && dist > 0.0001 {
                    let dir = delta / dist;
                    let factor = 1.0 - dist / MOUSE_INFLUENCE_RADIUS;
                    dir * (*strength) * factor
                } else {
                    Vec2::ZERO
                }
            }
            IOInteraction::Attract(strength) => {
                let delta = io_pos - particle_pos; // points toward IO
                let dist = delta.length();

                if dist < MOUSE_INFLUENCE_RADIUS && dist > 0.0001 {
                    let dir = delta / dist;
                    let factor = dist / MOUSE_INFLUENCE_RADIUS;
                    dir * (*strength) * factor
                } else {
                    Vec2::ZERO
                }
            }
        }
    }
}

impl Particles {
    pub fn new() -> Self {
        Self {
            pos: Vec::new(),
            predicted_pos: Vec::new(),
            vel: Vec::new(),
            density: Vec::new(),
            pressure: Vec::new(),
            force: Vec::new(),
        }
    }

    pub fn spawn(&mut self, particle: Particle) {
        self.pos.push(particle.pos);
        self.predicted_pos.push(particle.predicted_pos);
        self.vel.push(particle.vel);
        self.density.push(particle.density);
        self.pressure.push(particle.pressure);
        self.force.push(particle.force);
    }

    pub fn boundaries(world_size: Vec2, pos: &mut Vec2, vel: &mut Vec2) {
        let world_width = world_size.x;
        let world_height = world_size.y;

        let particle_radius_m = RADIUS; // / SCALE;

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

    pub fn integrate(
        &mut self,
        world_size: Vec2,
        mouse_pos: Vec2,
        interaction: IOInteraction,
        dt: f32,
    ) {
        for i in 0..NO_PARTICLES {
            let acceleration = self.force[i] / self.density[i];

            self.vel[i] += acceleration * dt;

            let interaction_vel = interaction.delta_vel(self.pos[i], mouse_pos);
            self.vel[i] += interaction_vel;

            if self.vel[i].length_squared() > MAX_VEL * MAX_VEL {
                self.vel[i] = (self.vel[i] / self.vel[i].length()) * MAX_VEL;
            }

            self.pos[i] += self.vel[i] * dt;
            Self::boundaries(world_size, &mut self.pos[i], &mut self.vel[i]);
        }
    }

    pub fn update(&mut self, dt: f32, world_size: Vec2) {
        let mut cells: Vec<(u32, usize)> = Vec::new(); //cell id, particle id
        let grid_width = (world_size.x / CELL_SIZE).floor() as usize;
        let grid_height = (world_size.y / CELL_SIZE).floor() as usize;
        let total_cells = grid_width * grid_height;

        for i in 0..self.pos.len() {
            self.predicted_pos[i] = self.pos[i] + self.vel[i] * dt;
            Self::boundaries(world_size, &mut self.predicted_pos[i], &mut self.vel[i]);
            let grid_coord = search::grid_coord(self.predicted_pos[i]);
            cells.push((search::hash(grid_coord, world_size), i));
        }
        cells.sort_by_key(|k| k.0);

        let mut lookups = vec![(0usize, 0usize); total_cells];

        for (i, &(cell_id, _particle_id)) in cells.iter().enumerate() {
            let data = &mut lookups[cell_id as usize];

            if data.1 == 0 {
                data.0 = i;
            }

            data.1 += 1;
        }

        self.density
            .par_iter_mut()
            .enumerate()
            .zip(self.pressure.par_iter_mut())
            .for_each(|((i, density_ref), pressure_ref)| {
                let mut current_density: f32 = 0.0;
                let grid_coord = search::grid_coord(self.predicted_pos[i]);
                let grid_neighbours = search::neighbours();

                for (offset_x, offset_y) in grid_neighbours {
                    let neighbor_x = grid_coord.x as i32 + offset_x;
                    let neighbor_y = grid_coord.y as i32 + offset_y;
                    if neighbor_x >= 0
                        && neighbor_x < grid_width as i32
                        && neighbor_y >= 0
                        && neighbor_y < grid_height as i32
                    {
                        let valid_coord = uvec2(neighbor_x as u32, neighbor_y as u32);
                        let cell_key = search::hash(valid_coord, world_size);

                        let (start_index, count) = lookups[cell_key as usize];

                        for j in 0..count {
                            let particle_idx = cells[start_index + j].1;

                            current_density += calculate_density(
                                self.predicted_pos[i],
                                self.predicted_pos[particle_idx],
                            );
                        }
                    }
                }
                // for j in 0..NO_PARTICLES {
                //     current_density +=
                //         calculate_density(self.predicted_pos[i], self.predicted_pos[j]);
                // }
                *density_ref = current_density;
                *pressure_ref = calculate_pressure(*density_ref);
            });

        let predicted_pos = &self.predicted_pos;
        let pressures = &self.pressure;
        let densities = &self.density;

        self.force
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, force_ref)| {
                let mut current_force = Vec2::ZERO;
                // println!("{}", densities[i]);
                let grid_coord = search::grid_coord(self.predicted_pos[i]);
                let grid_neighbours = search::neighbours();
                for (offset_x, offset_y) in grid_neighbours {
                    let neighbor_x = grid_coord.x as i32 + offset_x;
                    let neighbor_y = grid_coord.y as i32 + offset_y;
                    if neighbor_x >= 0
                        && neighbor_x < grid_width as i32
                        && neighbor_y >= 0
                        && neighbor_y < grid_height as i32
                    {
                        let valid_coord = uvec2(neighbor_x as u32, neighbor_y as u32);
                        let cell_key = search::hash(valid_coord, world_size);

                        let (start_index, count) = lookups[cell_key as usize];

                        for j in 0..count {
                            let particle_idx = cells[start_index + j].1;

                            if i == particle_idx {
                                continue;
                            }
                            let pressure_force = calculate_pressure_force(
                                predicted_pos[i],
                                predicted_pos[particle_idx],
                                pressures[i],
                                pressures[particle_idx],
                                densities[particle_idx],
                            );
                            current_force -= pressure_force;
                        }
                    }
                }
                // for j in 0..NO_PARTICLES {
                //     if i == j {
                //         continue;
                //     }
                //
                //     let pressure_force = calculate_pressure_force(
                //         predicted_pos[i],
                //         predicted_pos[j],
                //         pressures[i],
                //         pressures[j],
                //         densities[j],
                //     );
                //
                //     current_force -= pressure_force;
                // }

                let gravity_force = calculate_gravity_force(densities[i]);
                current_force += gravity_force;

                *force_ref = current_force;
            });
    }
}
