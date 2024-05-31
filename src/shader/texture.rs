use parking_lot::RwLock;
use std::{fmt::Debug, sync::Arc};

use iced::widget::shader::wgpu;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub size: wgpu::Extent3d,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Texture {
    pub fn new(device: &wgpu::Device, size: (u32, u32), label: Option<&str>) -> Self {
        let (width, height) = size;

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb, // srgb or no srgb
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture,
            size,
            bind_group,
            bind_group_layout,
        }
    }

    /// Upload texture to gpu
    ///
    /// TODO: only upload texture to gpu if changed
    pub fn upload(&mut self, queue: &wgpu::Queue, pixmap: &Pixmap) {
        pixmap.read(|texture| {
            let PixmapRef {
                buffer,
                width,
                height,
            } = texture;

            queue.write_texture(
                self.texture.as_image_copy(),
                bytemuck::cast_slice(buffer),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                self.size,
            );
        });
    }
}

pub struct PixmapRef<'a> {
    pub buffer: &'a [u32],
    pub width: u32,
    pub height: u32,
}

pub struct PixmapMut<'a> {
    pub buffer: &'a mut [u32],
    pub width: u32,
    pub height: u32,
}

/// RGBA pixmap shared between the cpu and gpu
#[derive(Clone)]
pub struct Pixmap {
    buffer: Arc<RwLock<Box<[u32]>>>,
    width: u32,
    height: u32,
}

impl Debug for Pixmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pixmap")
            .field("buffer", &"...")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Pixmap {
    /// creates and preallocates an empty pixmap
    pub(crate) fn new(width: u32, height: u32) -> Self {
        let buffer = vec![0; width as usize * height as usize].into_boxed_slice();

        Self {
            buffer: Arc::new(RwLock::new(buffer)),
            width,
            height,
        }
    }

    pub fn write<F>(&self, f: F)
    where
        F: FnOnce(PixmapMut<'_>),
    {
        f(PixmapMut {
            buffer: self.buffer.write().as_mut(),
            width: self.width,
            height: self.height,
        })
    }

    pub fn read<T, F>(&self, get: F) -> T
    where
        F: FnOnce(PixmapRef<'_>) -> T,
    {
        get(PixmapRef {
            buffer: self.buffer.read().as_ref(),
            width: self.width,
            height: self.height,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
