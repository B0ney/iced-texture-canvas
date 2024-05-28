mod texture;

use glam::Vec2;
use iced::mouse;
use iced::widget::shader::{self, wgpu, Primitive};

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Uniforms {
    center: Vec2,
    scale: f32,
}

/// Idea - what if the pipeline gives us a handle to upload textures?
struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture: texture::Texture,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false }, // todo
                        },
                        count: None,
                    },
                    // todo
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,

                        // this should match the filterable field of the
                        // corresponding texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[todo!()], // texture buffer
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "vs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: todo!(),
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // let pipeline_layout = device.create_pipeline_layout(todo!());

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("texture uniform"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(todo!());

        Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            texture: todo!(),
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, texture: (), uniforms: &Uniforms) {
        //TODO: upload texture to gpu (if changed)
        // apply uniform buffer to transform, clip texture view.
    }

    pub fn render(
        &self,
        target: &wgpu::TextureView,
        viewport: &iced::Rectangle<u32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //     label: todo!(),
        //     color_attachments: todo!(),
        //     depth_stencil_attachment: todo!(),
        //     timestamp_writes: todo!(),
        //     occlusion_query_set: todo!(),
        // });

        // pass.set_pipeline(todo!());
        // pass.set_bind_group(0, todo!(), &[]);
        // pass.set_vertex_buffer(0, todo!());
    }
}

struct Bitmap {}

impl<Message> shader::Program<Message> for Bitmap {
    type State = ();

    type Primitive = BitmapPrimatrive;

    fn draw(
        &self,
        state: &Self::State,
        cursor: mouse::Cursor,
        bounds: iced::Rectangle,
    ) -> Self::Primitive {
        Self::Primitive::new()
    }
}

const ZOOM_PIXELS_FACTOR: f32 = 200.0;

#[derive(Debug, Default)]
struct BitmapPrimatrive {
    controls: Controls,
}

impl BitmapPrimatrive {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, Default)]
struct Controls {
    zoom: f32,
    center: Vec2,
}

impl Controls {
    fn scale(&self) -> f32 {
        1.0 / 2.0_f32.powf(self.zoom) / ZOOM_PIXELS_FACTOR
    }
}

impl shader::Primitive for BitmapPrimatrive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        _bounds: &iced::Rectangle,
        _viewport: &shader::Viewport,
    ) {
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(device, format));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        let texture = ();

        pipeline.update(
            queue,
            texture,
            &Uniforms {
                center: self.controls.center,
                scale: self.controls.scale(),
            },
        );

        // let render_pipeline_layout =
        //     device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //         label: Some("Render Pipeline Layout"),
        //         bind_group_layouts: &[todo!()], // texture & camera bind group layout
        //         push_constant_ranges: &[],
        //     });

        // let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //     label: Some("Render Pipeline"),
        //     layout: Some(todo!()),
        //     vertex: todo!(),
        //     primitive: todo!(),
        //     depth_stencil: todo!(),
        //     multisample: todo!(),
        //     fragment: todo!(),
        //     multiview: todo!(),
        // });

        // todo: in our prepare stage, we upload the texture
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        let pipeline = storage.get::<Pipeline>().unwrap();
        pipeline.render(target, clip_bounds, encoder);
    }
}
