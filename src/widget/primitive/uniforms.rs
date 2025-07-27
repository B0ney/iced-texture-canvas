use glam::Vec2;
use iced_core::Size;
use iced_wgpu::wgpu;

pub struct Uniform {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Uniform {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform"),
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

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
#[repr(C)]
pub struct UniformsRaw {
    pub transform: [f32; 16],
}

impl UniformsRaw {
    pub fn new(center: Vec2, zoom: f32, screen: Size<f32>, texture: Size<f32>) -> Self {
        let (width, height) = (screen.width, screen.height);

        let position = center;
        let projection = screen_to_mat(0.0, width, height, 0.);

        let scale = glam::Vec3::new(texture.width, texture.height, 0.0);
        let pos = glam::Vec3::new(position.x, position.y, 0.0);

        let transform = glam::Mat4::from_translation(pos)
            * glam::Mat4::from_scale(scale)
            * glam::Mat4::from_scale(glam::Vec3::new(zoom, zoom, 0.0));

        UniformsRaw {
            transform: *(projection * transform).as_ref(),
        }
    }
}

fn screen_to_mat(left: f32, right: f32, bottom: f32, up: f32) -> glam::Mat4 {
    glam::Mat4::orthographic_rh(left, right, bottom, up, 0., 1.)
}
