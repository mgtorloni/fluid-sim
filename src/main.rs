mod constants;
mod gpu;
use std::sync::Arc;

use crate::constants::SimulationParams;
use crate::gpu::context::GpuContext;
use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

pub struct App {
    gpu_context: Option<GpuContext>,
    window: Option<Arc<Window>>,
    last_render_time: std::time::Instant,
    frame_rate: u32,
    params: SimulationParams,
}

impl Default for App {
    fn default() -> Self {
        Self {
            gpu_context: None,
            window: None,
            last_render_time: std::time::Instant::now(),
            frame_rate: 0,
            params: SimulationParams::default(),
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
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
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
                if let Some(gpu) = &mut self.gpu_context {
                    gpu.compute(self.params.no_particles);
                    match gpu.render(self.params.no_particles) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            gpu.resize(gpu.size);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
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
