use crate::constants::SimulationParams;
use std::num::NonZeroU32;
use std::sync::Arc;
use wgpu::{self, util::DeviceExt};
use wgpu_sort;
use winit::window::Window;

use super::particle::GpuParticle;
use super::pipelines::Pipelines;

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub config: wgpu::SurfaceConfiguration,

    pub pipelines: Pipelines,
    pub constants_buffer: wgpu::Buffer,
    pub lookups_buffer: wgpu::Buffer,
    pub sorter: wgpu_sort::GPUSorter,
    pub sort_buffers: wgpu_sort::SortBuffers,
}

impl GpuContext {
    pub async fn new(window: Arc<Window>, params: SimulationParams) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find GPU adapter");

        println!("Using GPU: {:?}", adapter.get_info().name);
        println!("Using Backend: {:?}", adapter.get_info().backend);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("GPU Device"),
                required_features: wgpu::Features::VERTEX_WRITABLE_STORAGE,
                ..Default::default()
            })
            .await
            .expect("Failed to open GPU device");
        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1), // wgpu crashes if width/height are 0
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoNoVsync,
            // present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let initial_particles = GpuParticle::spawn_particles(&params, config.width, config.height);
        let particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&initial_particles),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let constants_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Constants Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let sorter = wgpu_sort::GPUSorter::new(&device, 32);
        let sort_buffers =
            sorter.create_sort_buffers(&device, NonZeroU32::new(params.no_particles).unwrap());

        let grid_width = (params.width / params.cell_size).floor() as u32;
        let grid_height = (params.height / params.cell_size).floor() as u32;
        let lookups_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Lookups Buffer"),
            size: (grid_width * grid_height * 8) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let pipelines = Pipelines::new(
            &device,
            surface_format,
            &particle_buffer,
            &constants_buffer,
            &sort_buffers,
            &lookups_buffer,
        );
        Self {
            surface,
            device,
            queue,
            size,
            config,
            pipelines,
            constants_buffer,
            lookups_buffer,
            sort_buffers,
            sorter,
        }
    }

    pub fn update_params(&self, params: &SimulationParams) {
        self.queue
            .write_buffer(&self.constants_buffer, 0, bytemuck::cast_slice(&[*params]));
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn compute(&mut self, num_particles: u32) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });
        let workgroup_count = (num_particles as f32 / 64.0).ceil() as u32;

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Assign Cells Pass"),
                ..Default::default()
            });
            compute_pass.set_pipeline(&self.pipelines.hash);
            compute_pass.set_bind_group(0, &self.pipelines.bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        self.sorter
            .sort(&mut encoder, &self.queue, &self.sort_buffers, None);

        encoder.clear_buffer(&self.lookups_buffer, 0, None);

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Calculate Lookups Pass"),
                ..Default::default()
            });
            compute_pass.set_pipeline(&self.pipelines.lookups);
            compute_pass.set_bind_group(0, &self.pipelines.bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Density Compute Pass"),
                ..Default::default()
            });
            compute_pass.set_pipeline(&self.pipelines.density);
            compute_pass.set_bind_group(0, &self.pipelines.bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Forces Compute Pass"),
                ..Default::default()
            });
            compute_pass.set_pipeline(&self.pipelines.forces);
            compute_pass.set_bind_group(0, &self.pipelines.bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Physics Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.pipelines.physics);
            compute_pass.set_bind_group(0, &self.pipelines.bind_group, &[]);
            let workgroup_count = (num_particles as f32 / 64.0).ceil() as u32;
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
    pub fn render(&mut self, num_particles: u32) -> Result<(), wgpu::CurrentSurfaceTexture> {
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            error => return Err(error),
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.pipelines.render);
            render_pass.set_bind_group(0, &self.pipelines.bind_group, &[]);

            // Draw 6 vertices (1 square) per particle!
            render_pass.draw(0..6, 0..num_particles);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
