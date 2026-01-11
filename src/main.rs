mod constants;
mod engine;

use crate::constants::{MOUSE_FORCE_STRENGTH, REST_DENSITY, SCALE};
use crate::engine::renderer::FluidRenderer;
use crate::engine::simulation::{Particle, Particles};
use macroquad::prelude::*;

fn conf() -> Conf {
    Conf {
        window_title: "fluidsim".to_owned(),
        window_height: 700,
        window_width: 700,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut simulation = Particles::new();
    let renderer = FluidRenderer::new();

    let cols = 20;
    let rows = 40;
    let spacing = 0.1; // 10cm spacing

    let grid_width = cols as f32 * spacing;
    let grid_height = rows as f32 * spacing;
    //
    let world_size = vec2(screen_width() / SCALE, screen_height() / SCALE);
    //
    let offset_x = (world_size.x - grid_width) / 2.0;
    let offset_y = (world_size.y - grid_height) / 2.0;

    for y in 0..rows {
        for x in 0..cols {
            simulation.spawn(Particle {
                pos: vec2(offset_x + x as f32 * spacing, offset_y + y as f32 * spacing),
                vel: vec2(0.0, 0.0),
                density: REST_DENSITY,
                pressure: 0.0,
                force: vec2(0.0, 0.0),
            });
        }
    }

    // for _ in 0..NO_PARTICLES {
    //     simulation.spawn(Particle {
    //         pos: vec2(
    //             rand::gen_range(0.0, world_size.x),
    //             rand::gen_range(0.0, world_size.y),
    //         ),
    //         vel: vec2(0.0, 0.0),
    //         density: REST_DENSITY,
    //         pressure: 0.0,
    //         force: vec2(0.0, 0.0),
    //     });
    // }
    const PHYSICS_DT: f32 = 0.002;
    let mut accumulator = 0.0;

    let mut previous_time = get_time();

    loop {
        let current_time = get_time();
        let frame_time = current_time - previous_time;
        previous_time = current_time;

        accumulator += frame_time.min(0.002) as f32;
        let (mx, my) = mouse_position();
        let mouse_world_pos = vec2(mx / SCALE, my / SCALE);

        //TODO: Put this in an enum?
        let interaction_strength = if is_mouse_button_down(MouseButton::Left) {
            MOUSE_FORCE_STRENGTH
        } else if is_mouse_button_down(MouseButton::Right) {
            -MOUSE_FORCE_STRENGTH
        } else {
            0.0
        };

        while accumulator >= PHYSICS_DT {
            simulation.update();
            simulation.integrate(
                world_size,
                mouse_world_pos,
                interaction_strength,
                PHYSICS_DT,
            );
            accumulator -= PHYSICS_DT;
        }

        clear_background(BLACK);
        renderer.draw(&simulation);

        draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 30.0, WHITE);

        next_frame().await;
    }
}
