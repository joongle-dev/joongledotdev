use wgpu::{RenderPipelineDescriptor, ShaderSource, TextureDescriptor};
use wgpu::util::DeviceExt;

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    transforms_buffer: wgpu::Buffer,
    transforms_bind_group: wgpu::BindGroup,
    _depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    material_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl Renderer {
    pub async fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        let canvas_width = canvas.width();
        let canvas_height = canvas.height();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = match instance.create_surface_from_canvas(canvas) {
            Ok(surface) => surface,
            Err(error) => panic!("Failed to create surface from canvas: {error}")
        };

        let adapter = match instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: Some(&surface),
                force_fallback_adapter: true,
            }
        ).await {
            Some(adapter) => adapter,
            None => panic!("Failed render adapter request.")
        };
        let adapter_info = adapter.get_info();
        log::info!("Graphics adapter name: {}", adapter_info.name);
        log::info!("Graphics adapter driver: {}", adapter_info.driver);
        log::info!("Graphics adapter driver info: {}", adapter_info.driver_info);

        let (device, queue) = match adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_webgl2_defaults(),
                label: Some("Graphics Device"),
            },
            None
        ).await {
            Ok(device_queue) => device_queue,
            Err(error) => panic!("Failed render device request: {error}")
        };

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats.iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: canvas_width,
            height: canvas_height,
            present_mode: surface_capabilities.present_modes[0], //TODO: Present mode setting.
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![]
        };
        surface.configure(&device, &config);

        const TRANSFORM_SIZE: u64 = 4 * 16;
        const CAMERA_BUFFER_SIZE: u64 = TRANSFORM_SIZE;
        const TRANSFORMS_BUFFER_SIZE: u64 = TRANSFORM_SIZE * 16;

        let shader_module = device.create_shader_module(
            wgpu::ShaderModuleDescriptor{
                label: Some("Shader Module"),
                source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            }
        );
        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor{
                label: Some("Camera Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry{
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(CAMERA_BUFFER_SIZE),
                        },
                        count: None,
                    }
                ], 
            }
        );
        let transform_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor{
                label: Some("Transform Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry{
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(TRANSFORMS_BUFFER_SIZE),
                        },
                        count: None,
                    }
                ], 
            }
        );
        let material_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor{ 
                label: Some("Material Bind Group Layout"), 
                entries: &[
                    wgpu::BindGroupLayoutEntry{ 
                        binding: 0, 
                        visibility: wgpu::ShaderStages::FRAGMENT, 
                        ty: wgpu::BindingType::Texture { 
                            sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                            view_dimension: wgpu::TextureViewDimension::D2, 
                            multisampled: false, 
                        }, 
                        count: None, 
                    }, 
                    wgpu::BindGroupLayoutEntry{ 
                        binding: 1, 
                        visibility: wgpu::ShaderStages::FRAGMENT, 
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), 
                        count: None, 
                    }
                ], 
            }
        );
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor{
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &transform_bind_group_layout,
                    &material_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );
        let pipeline = device.create_render_pipeline(
            &RenderPipelineDescriptor{
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[
                        wgpu::VertexBufferLayout{
                            array_stride: 32,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttribute{
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 0,
                                    shader_location: 0,
                                },
                                wgpu::VertexAttribute{
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 12,
                                    shader_location: 1,
                                },
                                wgpu::VertexAttribute{
                                    format: wgpu::VertexFormat::Float32x2,
                                    offset: 24,
                                    shader_location: 2,
                                }
                            ],
                        }
                    ],
                },
                primitive: wgpu::PrimitiveState{
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: Some(wgpu::IndexFormat::Uint32),
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState{
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState{
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[
                        Some(wgpu::ColorTargetState{
                            format: surface_format,
                            blend: Some(wgpu::BlendState{
                                color: Default::default(),
                                alpha: Default::default()
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })
                    ],
                }),
                multiview: None,
            }
        );

        let camera_buffer = device.create_buffer(
            &wgpu::BufferDescriptor{
                label: Some("Camera Buffer"),
                size: TRANSFORM_SIZE,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor{
                label: Some("Camera Bind Group"),
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry{
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding{
                                buffer: &camera_buffer,
                                offset: 0,
                                size: None
                            }
                        )
                    }
                ],
            }
        );
        let transforms_buffer = device.create_buffer(
            &wgpu::BufferDescriptor{
                label: Some("Transforms Buffer"),
                size: TRANSFORM_SIZE * 16,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        let transforms_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor{
                label: Some("Transforms Bind Group"),
                layout: &transform_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry{
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding{
                                buffer: &transforms_buffer,
                                offset: 0,
                                size: None
                            }
                        )
                    }
                ],
            }
        );
        let depth_texture = device.create_texture(
            &wgpu::TextureDescriptor{
                label: Some("Depth Texture"),
                size: wgpu::Extent3d{
                    width: canvas_width,
                    height: canvas_height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24Plus,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            }
        );
        let depth_texture_view = depth_texture.create_view(
            &wgpu::TextureViewDescriptor{
                label: Some("Depth Texture View"),
                format: Some(wgpu::TextureFormat::Depth24Plus),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            }
        );

        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                label: Some("Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 0.0,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            }
        );

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            camera_buffer,
            camera_bind_group,
            transforms_buffer,
            transforms_bind_group,
            _depth_texture: depth_texture,
            depth_texture_view,
            material_bind_group_layout,
            sampler,
        }
    }

    pub fn create_mesh(&self, vertices: &[super::Vertex], indices: &[u32]) -> super::Mesh {
        let vertex_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
        let index_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }
        );
        super::Mesh {
            vertices: vertex_buffer,
            indices: index_buffer,
            num_indices: indices.len() as u32,
        }
    }

    pub fn create_texture(&self, image: super::Image) -> super::Texture {
        let texture = self.device.create_texture_with_data(
            &self.queue,
            &TextureDescriptor{
                label: Some("Texture"),
                size: wgpu::Extent3d{
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 0,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D1,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            image.bytes()
        );
        let view = texture.create_view(
            &wgpu::TextureViewDescriptor{
                label: Some("Texture View"),
                format: None,
                dimension: None,
                aspect: Default::default(),
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            }
        );
        super::Texture {
            texture,
            view,
        }
    }

    pub fn create_material()

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.180392,
                            g: 0.301960,
                            b: 0.243137,
                            a: 1.000000,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}