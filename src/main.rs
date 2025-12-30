use macroquad::prelude::*;
use std::f32::consts::PI;

const RADIUS: f32 = 6.0;
const PPM: f32 = 20.0; // pixels per metre, play with this value. This value makes sense right now
const GRAVITY: f32 = 0.0 * PPM;
const DAMPING: f32 = 0.5;
const WALL_PRESSURE_FORCE: f32 = 200.0;
const INFLUENCE_RADIUS: f32 = 50.0;
const MASS: f32 = 1.0;
const GAS_CONSTANT: f32 = 100000.0;
const REST_DENSITY: f32 = 1.0;

struct Particles {
    pos: Vec<Vec2>,
    vel: Vec<Vec2>,
    density: Vec<f32>,
    pressure: Vec<f32>,
    force: Vec<Vec2>,
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

    fn spawn(
        &mut self,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        rho: f32,
        pressure: f32,
        fx: f32,
        fy: f32,
    ) {
        self.pos.push(vec2(x, y));
        self.vel.push(vec2(vx, vy));
        self.density.push(rho);
        self.pressure.push(pressure);
        self.force.push(vec2(fx, fy));
    }
    fn spiky_kernel_gradient(&mut self, i: usize, j: usize) -> Vec2 {
        // Used for pressure force calculations
        // (-45/(pi*h⁶)) * (h-r)² * r̂ if 0<=r<=h
        // 0 if h<r
        let delta = self.pos[i] - self.pos[j];
        let r = delta.length(); // magnitude of the vector pointing at particle i

        if r < 0.00001 {
            // particles can be in the same position in which case send them in random direction
            let theta = rand::gen_range(0.0, 2.0 * PI);

            let random_dir = vec2(theta.cos(), theta.sin());

            return (-45.0 / (PI * INFLUENCE_RADIUS.powf(6.0)))
                * (INFLUENCE_RADIUS).powf(2.0)
                * random_dir;
        }
        let r_hat = delta / r;
        if r <= INFLUENCE_RADIUS {
            (-45.0 / (PI * INFLUENCE_RADIUS.powf(6.0))) * (INFLUENCE_RADIUS - r).powf(2.0) * r_hat
        } else {
            vec2(0.0, 0.0)
        }
    }

    fn poly_kernel(&mut self, i: usize, j: usize) -> f32 {
        // used for density
        //(315 / (64πh⁹)) * (h² - r²)³  if r <= h
        // 0 if r>h
        let delta = self.pos[i] - self.pos[j];
        let r = delta.length(); // magnitude of the vector pointing at particle i
        if r <= INFLUENCE_RADIUS {
            (315.0 / (64.0 * PI * INFLUENCE_RADIUS.powf(9.0)))
                * (INFLUENCE_RADIUS.powf(2.0) - r.powf(2.0)).powf(3.0)
        } else {
            0.0
        }
    }

    fn update(&mut self) {
        let dt = get_frame_time();
        let count = self.pos.len();
        for i in 0..count {
            self.vel[i].y += GRAVITY * dt; // v = u + at
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
        let count = self.pos.len();
        for i in 0..count {
            self.density[i] = 0.0;
            self.pressure[i] = 0.0;
            self.force[i] = vec2(0.0, 0.0);
            for j in 0..count {
                self.density[i] += MASS * self.poly_kernel(i, j);
            }
            self.pressure[i] += GAS_CONSTANT * (self.density[i] - REST_DENSITY);
        }
        for i in 0..count {
            for j in 0..count {
                if i == j {
                    //NOTE: if i=j then the distance between the "two" particles
                    //is 0 and that grad_spiky will be NaN causing force calculation to fail
                    continue;
                }
                let grad_spiky = self.spiky_kernel_gradient(i, j);

                self.force[i] += MASS
                    * ((self.pressure[i] + self.pressure[j]) / (2.0 * self.density[j]))
                    * grad_spiky;
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
        window_height: 900,
        window_width: 1200,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut simulation = Particles::new();

    for _ in 0..400 {
        simulation.spawn(
            rand::gen_range(0.0, screen_width()),
            rand::gen_range(0.0, screen_height()),
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            // rand::gen_range(-50.0, 50.0),
        );
    }

    loop {
        clear_background(DARKGRAY);
        simulation.update();
        simulation.draw();

        next_frame().await;
    }
}
