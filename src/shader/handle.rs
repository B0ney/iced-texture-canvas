use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use iced_core::Size;

use crate::shader::SurfaceHandler;

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

    pub fn size(&self) -> Size {
        (self.width() as f32, self.height() as f32).into()
    }

    pub(crate) fn create_weak(&self) -> Weak<SurfaceInner> {
        Arc::downgrade(&self.0)
    }
}

impl SurfaceHandler for Surface {
    type Surface = SurfaceInner;

    fn width(&self) -> u32 {
        self.width()
    }

    fn height(&self) -> u32 {
        self.height()
    }

    fn create_weak(&self) -> Weak<Self::Surface> {
        self.create_weak()
    }
}

impl Clone for Surface {
    fn clone(&self) -> Self {
        Self(Arc::new(SurfaceInner::clone(&self.0)))
    }
}

pub struct SurfaceInner {
    buffer: Vec<u32>,
    width: u32,
    height: u32,
    dirty: AtomicBool,
}

impl super::Surface for SurfaceInner {
    fn width(&self) -> u32 {
        self.width()
    }

    fn height(&self) -> u32 {
        self.height()
    }

    fn data(&self) -> &[u8] {
        self.raw()
    }

    fn run_if_modified(&self, update: impl FnOnce(u32, u32, &[u8])) {
        if let Ok(true) =
            self.dirty
                .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
        {
            update(self.width, self.height, self.raw())
        }
    }
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

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> Size {
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

impl std::fmt::Debug for SurfaceInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("surface")
            .field("buffer", &"...")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}
