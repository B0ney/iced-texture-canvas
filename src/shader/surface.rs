use std::fmt::Debug;
use std::sync::{Arc, Weak};

/// A type that can provide information about the [`Surface`],
/// and create a weak reference to be used by the iced shader program.
pub trait SurfaceHandler {
    type Surface: Surface;

    /// The width of the [`Surface`]
    fn width(&self) -> u32;

    /// The height of the [`Surface`]
    fn height(&self) -> u32;

    /// Create a Weak reference to the [`Surface`].
    ///
    /// The program will then attempt to convert it to a strong reference
    /// at the prepare stage of the pipeline.
    ///
    /// A Weak reference grants users the freedom to modify their [`Surface`]
    /// without resorting to locks thanks to Arc::make_mut.
    fn create_weak(&self) -> Weak<Self::Surface>;
}

/// RGBA image data stored on the CPU to be uploaded to the GPU.
pub trait Surface: Send + Sync + Debug + 'static {
    /// The width of the [`Surface`]
    fn width(&self) -> u32;

    /// The height of the [`Surface`]
    fn height(&self) -> u32;

    /// The image data of [`Surface`]
    fn data(&self) -> &[u8];

    /// The size of the [`Surface`]
    fn size(&self) -> iced_core::Size {
        (self.width() as f32, self.height() as f32).into()
    }

    /// Call the update closure if the [`Surface`] was modified, or if `other` is true.
    ///
    /// The data provided in update will be uploaded to the GPU.
    fn run_if_modified(&self, update: impl FnOnce(u32, u32, &[u8]));
}

impl<T: Surface> Surface for Arc<T> {
    fn width(&self) -> u32 {
        Arc::as_ref(&self).width()
    }

    fn height(&self) -> u32 {
        Arc::as_ref(&self).height()
    }

    fn data(&self) -> &[u8] {
        Arc::as_ref(&self).data()
    }

    fn run_if_modified(&self, update: impl FnOnce(u32, u32, &[u8])) {
        Arc::as_ref(&self).run_if_modified(update)
    }
}
