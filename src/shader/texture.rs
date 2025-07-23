use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use iced::wgpu;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub size: wgpu::Extent3d,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Texture {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
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
            label: None,
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

    pub fn upload(&mut self, queue: &wgpu::Queue, pixmap: &PixmapRef) {
        queue.write_texture(
            self.texture.as_image_copy(),
            bytemuck::cast_slice(&pixmap.buffer),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * pixmap.width),
                rows_per_image: Some(pixmap.height),
            },
            self.size,
        );
    }
}

pub struct Surface(pub(crate) Arc<SurfaceInner>);

impl Surface {
    pub fn new(width: u32, height: u32) -> Self {
        let buffer = vec![0; width as usize * height as usize];

        Self(Arc::new(SurfaceInner {
            buffer,
            width,
            height,
            dirty: AtomicBool::new(false),
        }))
    }

    pub fn raw(&self) -> &[u8] {
        bytemuck::cast_slice(self.buffer())
    }

    pub fn buffer(&self) -> &[u32] {
        &self.0.buffer
    }

    pub fn width(&self) -> u32 {
        self.0.width
    }

    pub fn height(&self) -> u32 {
        self.0.height
    }

    pub fn raw_mut(&mut self) -> &mut [u8] {
        bytemuck::cast_slice_mut(self.buffer_mut())
    }

    pub fn buffer_mut(&mut self) -> &mut [u32] {
        Arc::make_mut(&mut self.0).buffer_mut()
    }

    pub fn update(&mut self, data: &[u8]) {
        self.raw_mut().copy_from_slice(data);
    }

    pub fn size(&self) -> iced::Size {
        (self.width() as f32, self.height() as f32).into()
    }

    pub fn pixmap_ref(&self) -> PixmapRef {
        PixmapRef {
            buffer: self.buffer(),
            width: self.width(),
            height: self.height(),
        }
    }

    pub fn pixmap_mut(&mut self) -> PixmapMut {
        PixmapMut {
            width: self.width(),
            height: self.height(),
            buffer: self.buffer_mut(),
        }
    }

    pub(crate) fn create_weak(&self) -> Weak<SurfaceInner> {
        Arc::downgrade(&self.0)
    }
}

impl Clone for Surface {
    fn clone(&self) -> Self {
        Self(Arc::new(SurfaceInner::clone(&self.0)))
    }
}

pub(crate) struct SurfaceInner {
    buffer: Vec<u32>,
    width: u32,
    height: u32,
    dirty: AtomicBool,
}

impl SurfaceInner {
    pub fn raw_mut(&mut self) -> &mut [u8] {
        bytemuck::cast_slice_mut(self.buffer_mut())
    }

    pub fn raw(&self) -> &[u8] {
        bytemuck::cast_slice(&self.buffer())
    }

    pub fn buffer_mut(&mut self) -> &mut [u32] {
        self.dirty.store(true, Ordering::Relaxed);
        &mut self.buffer
    }

    pub fn buffer(&self) -> &[u32] {
        &self.buffer
    }

    pub fn run_if_modified_or(&self, other: bool, update: impl FnOnce(PixmapRef)) {
        if other
            || self
                .dirty
                .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
                == Ok(true)
        {
            update(PixmapRef {
                buffer: &self.buffer,
                width: self.width,
                height: self.height,
            })
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> iced::Size {
        (self.width as f32, self.height as f32).into()
    }
}

impl Clone for SurfaceInner {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            width: self.width,
            height: self.height,
            dirty: AtomicBool::new(self.dirty.load(Ordering::Relaxed)),
        }
    }
}

impl Debug for SurfaceInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("surface")
            .field("buffer", &"...")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

#[derive(Clone, Copy)]
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
