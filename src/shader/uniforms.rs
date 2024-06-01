use glam::Vec2;
use iced::widget::shader::wgpu;

pub struct Uniform {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Uniform {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("controls uniform"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: std::mem::size_of::<UniformsRaw>() as u64,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    /// Upload uniform buffer
    pub fn upload(&self, queue: &wgpu::Queue, uniforms: UniformsRaw) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniforms]))
    }
}

/// TODO: make uniform just the transformation matrix
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
#[repr(C)]
pub struct UniformsRaw {
    pub center: Vec2,
    pub _padding: [f32; 2],
    pub matrix: [f32; 16],
}

impl UniformsRaw {
    pub fn new(center: Vec2, zoom: f32, screen: iced::Size<f32>, texture: iced::Size<f32>) -> Self {
        let aspect_ratio = (screen.width * texture.height) / (screen.height * texture.width);

        let projection =
            glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 1.0, 100.0);

        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(0.0, 0.0, 1.0),
            glam::Vec3::ZERO,
            glam::Vec3::Y,
        );

        UniformsRaw {
            center: center,
            _padding: Default::default(),
            matrix: *(projection * view * zoom).as_ref(),
        }
    }
}
