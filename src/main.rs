mod constants;
mod engine;
mod graphics;

use crate::constants::SimulationParams;

use crate::constants::{HEIGHT, NO_PARTICLES, WIDTH};
use crate::engine::simulation::{IOInteraction, Particle, Particles};
use crate::graphics::renderer::FluidRenderer;
use egui_macroquad::egui;
use macroquad::prelude::*;

fn conf() -> Conf {
    Conf {
        window_title: "fluidsim".to_owned(),
        window_height: HEIGHT,
        window_width: WIDTH,
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

    let world_size = vec2(screen_width(), screen_height());

    let offset_x = (world_size.x - grid_width) / 2.0;
    let offset_y = (world_size.y - grid_height) / 2.0;

    let mut params = SimulationParams::default();
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
                density: params.rest_density,
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
    loop {
        let dt = get_frame_time() / 10.0;
        let world_size = vec2(screen_width(), screen_height());
        let (mx, my) = mouse_position();
        let mouse_pos = vec2(mx, my);
        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Settings").show(egui_ctx, |ui| {
                params.ui(ui);
            });
        });

        let mouse_captured = egui_macroquad::egui::Context::default().wants_pointer_input();

        let interaction_strength = if !mouse_captured && is_mouse_button_down(MouseButton::Left) {
            IOInteraction::Repel(params.mouse_force)
        } else if !mouse_captured && is_mouse_button_down(MouseButton::Right) {
            IOInteraction::Attract(params.mouse_force)
        } else {
            IOInteraction::None
        };

        clear_background(BLACK);
        simulation.update(dt, world_size, &params);
        simulation.integrate(world_size, mouse_pos, interaction_strength, dt, &params);
        renderer.draw(&simulation, &params);

        draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 30.0, WHITE);
        egui_macroquad::draw();

        next_frame().await;
    }
}
