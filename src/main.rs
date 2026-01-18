mod constants;
mod engine;
mod graphics;

use crate::constants::{MOUSE_FORCE_STRENGTH, NO_PARTICLES, REST_DENSITY}; // SCALE};
use crate::engine::simulation::{IOInteraction, Particle, Particles};
use crate::graphics::renderer::FluidRenderer;
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

    let cols = (NO_PARTICLES as f32).sqrt().ceil() as usize;
    let rows = NO_PARTICLES.div_ceil(cols);
    let spacing = 10.0;

    let grid_width = cols as f32 * spacing;
    let grid_height = rows as f32 * spacing;

    // let world_size = vec2(screen_width() / SCALE, screen_height() / SCALE);
    let world_size = vec2(screen_width(), screen_height());

    let offset_x = (world_size.x - grid_width) / 2.0;
    let offset_y = (world_size.y - grid_height) / 2.0;

    let mut count = 0;
    for y in 0..rows {
        for x in 0..cols {
            if count >= NO_PARTICLES {
                break;
            }
            simulation.spawn(Particle {
                pos: vec2(offset_x + x as f32 * spacing, offset_y + y as f32 * spacing),
                predicted_pos: vec2(offset_x + x as f32 * spacing, offset_y + y as f32 * spacing),
                vel: vec2(0.0, 0.0),
                density: REST_DENSITY,
                pressure: 0.0,
                force: vec2(0.0, 0.0),
            });
            count += 1;
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

    loop {
        // let world_size = vec2(screen_width() / SCALE, screen_height() / SCALE);
        let world_size = vec2(screen_width(), screen_height());
        let (mx, my) = mouse_position();
        // let mouse_world_pos = vec2(mx / SCALE, my / SCALE);
        let mouse_world_pos = vec2(mx, my);

        let interaction_strength = if is_mouse_button_down(MouseButton::Left) {
            IOInteraction::Repel(MOUSE_FORCE_STRENGTH)
        } else if is_mouse_button_down(MouseButton::Right) {
            IOInteraction::Attract(MOUSE_FORCE_STRENGTH)
        } else {
            IOInteraction::None
        };

        clear_background(BLACK);
        simulation.update(PHYSICS_DT);
        simulation.integrate(
            world_size,
            mouse_world_pos,
            interaction_strength,
            PHYSICS_DT,
        );
        renderer.draw(&simulation);

        draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 30.0, WHITE);

        next_frame().await;
    }
}
