use wgpu::{self, PipelineCompilationOptions};
use wgpu_sort;

pub struct Pipelines {
    pub hash: wgpu::ComputePipeline,
    pub lookups: wgpu::ComputePipeline,
    pub density: wgpu::ComputePipeline,
    pub forces: wgpu::ComputePipeline,
    pub physics: wgpu::ComputePipeline,
    pub render: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        particle_buffer: &wgpu::Buffer,
        constants_buffer: &wgpu::Buffer,
        sort_buffers: &wgpu_sort::SortBuffers,
        lookups_buffer: &wgpu::Buffer,
    ) -> Pipelines {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &bind_group_layout,
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
                    resource: sort_buffers.keys().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: sort_buffers.values().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: lookups_buffer.as_entire_binding(),
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let search_shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/search.wgsl"));
        let update_shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/update.wgsl"));
        let render_shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/render.wgsl"));

        let hash = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Hash Pipeline"),
            layout: Some(&pipeline_layout),
            module: &search_shader,
            entry_point: Some("hash_particles"),
            cache: None,
            compilation_options: PipelineCompilationOptions::default(),
        });
        let lookups = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Lookups Pipeline"),
            layout: Some(&pipeline_layout),
            module: &search_shader,
            entry_point: Some("build_lookups"),
            cache: None,
            compilation_options: PipelineCompilationOptions::default(),
        });
        let density = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Density Pipeline"),
            layout: Some(&pipeline_layout),
            module: &update_shader,
            entry_point: Some("calculate_pressure_density"),
            cache: None,
            compilation_options: PipelineCompilationOptions::default(),
        });
        let forces = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Forces Pipeline"),
            layout: Some(&pipeline_layout),
            module: &update_shader,
            entry_point: Some("calculate_pressure_force"),
            cache: None,
            compilation_options: PipelineCompilationOptions::default(),
        });
        let physics = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Physics Pipeline"),
            layout: Some(&pipeline_layout),
            module: &update_shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: PipelineCompilationOptions::default(),
        });

        let render = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    format: surface_format,
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

        Pipelines {
            hash,
            lookups,
            density,
            forces,
            physics,
            render,
            bind_group,
        }
    }
}
