use crate::constants::SimulationParams;
use bytemuck::{Pod, Zeroable};
use rand::RngExt;
use std::sync::Arc;
use wgpu::{self, PipelineCompilationOptions, util::DeviceExt};
use winit::window::Window;

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub config: wgpu::SurfaceConfiguration,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub render_pipeline: wgpu::RenderPipeline,
    pub compute_bind_group: wgpu::BindGroup,
    pub constants_buffer: wgpu::Buffer,
    pub lookups_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuParticle {
    pub pos: [f32; 2],           // 8 bytes
    pub predicted_pos: [f32; 2], // 8 bytes
    pub vel: [f32; 2],           // 8 bytes
    pub force: [f32; 2],         // 8 bytes
    pub density: f32,            // 4 bytes
    pub pressure: f32,           // 4 bytes
                                 // we have 40 bytes
                                 // from specs (https://www.w3.org/TR/WGSL/#alignment-and-size) alignment of a struct is defined as
                                 // AlignOf(S) = max(AlignOfMember(S,0), max(AlignOfMember(S,1), ... , AlignOfMember(S,N)) = 8 here
                                 // SizeOf(S) = roundUp(AlignOf(S), justPastLastMember) =
                                 // ceil(justPastLastMember / AlignOf(S)) * AlignOf(S)
                                 // where justPastLastMember = OffsetOfMember(S,N) + SizeOfMember(S,N)

                                 // justPastLastMember = 36 + 4 = 40
                                 // since pressure starts at the 36th byte and is 4 bytes
                                 // 40 is divisible by 8 so roundUp(8, 40) = ceil(40/8) * 8 = 40
}

impl GpuContext {
    pub async fn new(window: Arc<Window>, params: SimulationParams) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
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
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        let mut initial_particles = Vec::with_capacity(params.no_particles as usize);

        // let cols = (NO_PARTICLES as f32).sqrt().ceil() as usize;
        // let spacing = 2.0; // Distance between particles
        // let start_x = 100.0;
        // let start_y = 100.0;
        // for i in 0..NO_PARTICLES {
        //     let x = (i % cols) as f32 * spacing + start_x;
        //     let y = (i / cols) as f32 * spacing + start_y;
        //
        //     initial_particles.push(GpuParticle {
        //         pos: [x, y],
        //         predicted_pos: [x, y],
        //         vel: [0.0, 0.0],
        //         force: [0.0, 0.0],
        //         density: 0.0,
        //         pressure: 0.0,
        //     });
        // }
        let mut rng = rand::rng();

        for _ in 0..params.no_particles {
            let (x, y) = (
                rng.random_range(0.0..=params.width),
                rng.random_range(0.0..=params.height),
            );
            initial_particles.push(GpuParticle {
                pos: [x, y],
                predicted_pos: [x, y],
                vel: [0.0, 0.0],
                density: params.rest_density,
                pressure: 0.0,
                force: [0.0, 0.0],
            });
        }
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
        let cells_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cells Buffer"),
            size: (params.no_particles * 8) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let grid_width = (params.width / params.cell_size).floor() as u32;
        let grid_height = (params.height / params.cell_size).floor() as u32;
        let total_cells = grid_width * grid_height;
        let lookups_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Lookups Buffer"),
            size: (total_cells * 8) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        surface.configure(&device, &config);
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: constants_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: cells_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: lookups_buffer.as_entire_binding(),
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            immediate_size: 0,
        });
        let update_shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/update.wgsl"));
        let render_shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/render.wgsl"));
        // let density_shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/density.wgsl");
        // let force_shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/force.wgsl");
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Update force and density Pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            module: &update_shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            compute_bind_group,
            compute_pipeline,
            render_pipeline,
            size,
            constants_buffer,
            lookups_buffer,
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
        encoder.clear_buffer(&self.lookups_buffer, 0, None);

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Physics Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            let workgroup_count = (num_particles as f32 / 64.0).ceil() as u32;
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
    pub fn render(&mut self, num_particles: u32) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
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
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.compute_bind_group, &[]);

            // Draw 6 vertices (1 square) per particle!
            render_pass.draw(0..6, 0..num_particles);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
