mod constants;
mod gpu;
use std::sync::Arc;

use crate::constants::SimulationParams;
use crate::gpu::context::GpuContext;
use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const MOUSE_FORCE: f32 = 200.0;

pub struct App {
    gpu_context: Option<GpuContext>,
    window: Option<Arc<Window>>,
    last_render_time: std::time::Instant,
    last_frame_time: std::time::Instant,
    frame_rate: u32,
    params: SimulationParams,
    attract_held: bool,
    repel_held: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            gpu_context: None,
            window: None,
            last_render_time: std::time::Instant::now(),
            last_frame_time: std::time::Instant::now(),
            frame_rate: 0,
            params: SimulationParams::default(),
            attract_held: false,
            repel_held: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes()
                .with_title("Fluid Simulation")
                .with_inner_size(winit::dpi::PhysicalSize::new(
                    self.params.width,
                    self.params.height,
                ));
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );
            self.window = Some(window.clone());
            let context = pollster::block_on(GpuContext::new(window.clone(), self.params));
            self.gpu_context = Some(context);
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let egui_wants_pointer =
            if let (Some(gpu), Some(window)) = (&mut self.gpu_context, &self.window) {
                gpu.handle_window_event(window, &event);
                gpu.egui_ctx.egui_wants_pointer_input()
            } else {
                false
            };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if !egui_wants_pointer {
                    let pressed = state == ElementState::Pressed;
                    match button {
                        MouseButton::Left => self.attract_held = pressed,
                        MouseButton::Right => self.repel_held = pressed,
                        _ => {}
                    }
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(gpu) = &mut self.gpu_context {
                    gpu.resize(physical_size);
                    self.params.width = physical_size.width as f32;
                    self.params.height = physical_size.height as f32;
                    gpu.update_params(&self.params);
                }
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let delta_time = (now - self.last_frame_time).as_secs_f32();
                self.last_frame_time = now;
                if let (Some(gpu), Some(window)) = (&mut self.gpu_context, self.window.as_ref()) {
                    self.params.mouse_strength = 0.0;
                    if self.attract_held || self.repel_held {
                        if let Some(pos) = gpu.egui_ctx.pointer_latest_pos() {
                            let scale = window.scale_factor() as f32;
                            self.params.mouse_pos = [pos.x * scale, pos.y * scale];
                            // attract takes precedence if both held
                            self.params.mouse_strength = if self.attract_held {
                                MOUSE_FORCE
                            } else {
                                -MOUSE_FORCE
                            };
                        }
                    }

                    let mut time_to_simulate = delta_time.min(0.1);

                    let max_step_dt = 1.0 / 120.0;
                    let max_substeps = 1;
                    let mut substeps = 0;
                    while time_to_simulate > 0.0 && substeps < max_substeps {
                        let step_dt = time_to_simulate.min(max_step_dt);
                        self.params.dt = step_dt;
                        gpu.update_params(&self.params);
                        gpu.compute(self.params.no_particles);
                        time_to_simulate -= step_dt;
                        substeps += 1;
                    }

                    let num_particles = self.params.no_particles;
                    let params = &mut self.params;
                    match gpu.render(
                        window,
                        |ctx| {
                            egui::Window::new("Parameters")
                                .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 8.0))
                                .resizable(false)
                                .show(ctx, |ui| {
                                    params.ui(ui);
                                });
                        },
                        num_particles,
                    ) {
                        Ok(_) => {}
                        Err(wgpu::CurrentSurfaceTexture::Lost)
                        | Err(wgpu::CurrentSurfaceTexture::Outdated) => {
                            gpu.resize(gpu.size);
                        }
                        Err(wgpu::CurrentSurfaceTexture::Occluded)
                        | Err(wgpu::CurrentSurfaceTexture::Timeout) => {}
                        Err(wgpu::CurrentSurfaceTexture::Validation) => {
                            event_loop.exit();
                        }
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                self.frame_rate += 1;
                let elapsed = self.last_render_time.elapsed().as_secs_f32();
                if elapsed >= 0.5 {
                    let fps = (self.frame_rate as f32 / elapsed).round() as u32;
                    if let Some(window) = &self.window {
                        window.set_title(&format!("Fluid Simulation | FPS: {}", fps));
                    }
                    self.frame_rate = 0;
                    self.last_render_time = std::time::Instant::now();
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
}

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app)
}
