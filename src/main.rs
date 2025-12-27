use macroquad::prelude::*;

const RADIUS: f32 = 4.0;
const PPM: f32 = 20.0; // pixels per metre, play with this value. This makes sense right now
const GRAVITY: f32 = 9.80 * PPM;
const RESTITUTION: f32 = 0.5;

struct Particles {
    pos: Vec<Vec2>,
    vel: Vec<Vec2>,
}
impl Particles {
    fn new() -> Self {
        Self {
            pos: Vec::new(),
            vel: Vec::new(),
        }
    }

    fn spawn(&mut self, x: f32, y: f32, vx: f32, vy: f32) {
        self.pos.push(vec2(x, y));
        self.vel.push(vec2(vx, vy));
    }

    fn collide(&mut self) {
        let count = self.pos.len();
        for i in 0..count {
            for j in i + 1..count {
                let delta_length = self.pos[i] - self.pos[j];
                let dist_sq = delta_length.length_squared();
                let min_dist = RADIUS * 2.0;

                if dist_sq < min_dist * min_dist {
                    // distance between two particles diameter, we collide particles
                    let dist = dist_sq.sqrt();
                    let normal = delta_length / dist;
                    let rel_vel = self.vel[i] - self.vel[j];
                    let vel_along_normal = rel_vel.dot(normal);
                    if vel_along_normal > 0.0 {
                        continue;
                    }

                    let impulse_mag = -(1.0 + RESTITUTION) * vel_along_normal / 2.0;

                    let impulse = normal * impulse_mag;
                    self.vel[i] += impulse;
                    self.vel[j] -= impulse;

                    let percent = 0.8;
                    let penetration = min_dist - dist;
                    let correction = normal * (penetration / 2.0) * percent;

                    self.pos[i] += correction;
                    self.pos[j] -= correction;
                }
            }
        }
    }

    fn update(&mut self) {
        let dt = get_frame_time();
        let count = self.pos.len();
        for i in 0..count {
            self.vel[i].y += GRAVITY * dt; // v = u + at
            self.pos[i] += self.vel[i] * dt; // s = u + vt

            if self.pos[i].x >= screen_width() - RADIUS {
                self.vel[i].x *= -RESTITUTION;
                self.pos[i].x = screen_width() - RADIUS;
            } else if self.pos[i].x <= RADIUS {
                self.vel[i].x = -self.vel[i].x * RESTITUTION;
                self.pos[i].x = RADIUS;
            }
            if self.pos[i].y >= screen_height() - RADIUS {
                self.vel[i].y = -self.vel[i].y * RESTITUTION;
                self.pos[i].y = screen_height() - RADIUS;
            }
        }
        self.collide();
    }
    fn draw(&self) {
        let max_speed: f32 = 100.0;

        for i in 0..self.pos.len() {
            let ratio: f32 = (self.vel[i].length() / max_speed).min(1.0);
            let colour = Color::new(
                (ratio * 2.0).min(1.0),
                1.0 - (ratio - 0.5).abs() * 2.0,
                ((1.0 - ratio) * 2.0).min(1.0),
                1.0,
            );
            draw_circle(self.pos[i].x, self.pos[i].y, RADIUS, colour);
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

    for _ in 0..1000 {
        simulation.spawn(
            rand::gen_range(0.0, screen_width()),
            rand::gen_range(0.0, 200.0),
            rand::gen_range(-50.0, 50.0),
            rand::gen_range(-50.0, 50.0),
        );
    }

    loop {
        clear_background(DARKGRAY);
        simulation.update();
        simulation.draw();

        next_frame().await
    }
}
