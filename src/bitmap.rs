//! A concrete implementation of the [`SurfaceHandler`] (and [`Surface`](crate::Surface)) in the form of a [`Bitmap`] for convenience.
use crate::widget::surface::SurfaceHandler;

use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use iced_core::Size;

/// Create an empty [`Bitmap`] image.
///
/// # Panics
///
/// Panics if either the width or height is zero.
pub fn bitmap(width: u32, height: u32) -> Bitmap {
    Bitmap::new(width, height)
}

/// Image data stored on the CPU that can be displayed by a [`TextureCanvas`](crate::TextureCanvas).
///
/// A [`Bitmap`] can be freely edited and resized.
/// 
/// **Note**: 
/// While it contains an [`Arc`], cloning this type will create a new allocation.
pub struct Bitmap(pub(crate) Arc<SurfaceInner>);

impl Bitmap {
    /// Create an empty [`Bitmap`] image.
    ///
    /// # Panics
    ///
    /// Panics if either the width or height is zero.
    pub fn new(width: u32, height: u32) -> Self {
        let buffer = vec![0; width as usize * height as usize];

        Self(Arc::new(SurfaceInner {
            buffer,
            width: NonZeroU32::new(width).expect("width must be greater than 0"),
            height: NonZeroU32::new(height).expect("height must be greater than 0"),
            dirty: AtomicBool::new(false),
        }))
    }

    /// Create a [`Bitmap`] image with initialized data.
    ///
    /// # Panics
    ///
    /// Panics if the `width` * `height` * `4` doesn't match the length of the data.
    pub fn new_init(width: u32, height: u32, data: &[u8]) -> Self {
        assert_eq!(
            width as usize * height as usize * 4,
            data.len(),
            "Size mismatch!"
        );

        let mut this = Self::new(width, height);
        this.update(data);
        this
    }

    /// Resize the [`Bitmap`].
    ///
    /// # Panics
    ///
    /// Panics if either the width or height is zero.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == self.width() && height == self.height() {
            return;
        }

        let this = Arc::make_mut(&mut self.0);

        this.buffer.resize(width as usize * height as usize, 0);
        this.width = NonZeroU32::new(width).expect("width must be greater than 0");
        this.height = NonZeroU32::new(height).expect("height must be greater than 0");
    }

    /// Get the width of the [`Bitmap`]
    pub fn width(&self) -> u32 {
        self.0.width.get()
    }

    /// Get the height of the [`Bitmap`]
    pub fn height(&self) -> u32 {
        self.0.height.get()
    }

    /// Get the [`Size`] of the [`Bitmap`]
    pub fn size(&self) -> Size {
        (self.width() as f32, self.height() as f32).into()
    }

    /// Get an immutable u8 slice of the raw `RGBA` image data.
    pub fn raw(&self) -> &[u8] {
        bytemuck::cast_slice(self.buffer())
    }

    /// Get a mutable u8 slice of the raw `RGBA` image data.
    pub fn raw_mut(&mut self) -> &mut [u8] {
        bytemuck::cast_slice_mut(self.buffer_mut())
    }

    /// Get an immutable u32 slice of the raw `RGBA` image data.
    pub fn buffer(&self) -> &[u32] {
        &self.0.buffer
    }

    /// Get a mutable u32 slice of the raw `RGBA` image data.
    pub fn buffer_mut(&mut self) -> &mut [u32] {
        Arc::make_mut(&mut self.0).buffer_mut()
    }

    /// Update the image buffer with the provided data.
    pub fn update(&mut self, data: &[u8]) {
        self.raw_mut().copy_from_slice(data);
    }

    pub(crate) fn create_weak(&self) -> Weak<SurfaceInner> {
        Arc::downgrade(&self.0)
    }
}

impl SurfaceHandler for Bitmap {
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

impl Clone for Bitmap {
    fn clone(&self) -> Self {
        Self(Arc::new(SurfaceInner::clone(&self.0)))
    }
}

pub struct SurfaceInner {
    buffer: Vec<u32>,
    width: NonZeroU32,
    height: NonZeroU32,
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
            update(self.width(), self.height(), self.raw())
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
        self.width.get()
    }

    pub fn height(&self) -> u32 {
        self.height.get()
    }

    pub fn size(&self) -> Size {
        (self.width() as f32, self.height() as f32).into()
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
