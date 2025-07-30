use super::State;

use iced_core::widget::{self, Id, Operation};

/// Create an [`Operation`] that will center the image given an [`Id`].
pub fn center_image_raw(id: Id) -> impl Operation + 'static {
    CenterImage { id: id.into() }
}

struct CenterImage {
    id: Id,
}

impl<T> Operation<T> for CenterImage {
    fn container(
        &mut self,
        _id: Option<&Id>,
        _bounds: iced_core::Rectangle,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
    ) {
        operate_on_children(self);
    }

    fn custom(
        &mut self,
        id: Option<&Id>,
        _bounds: iced_core::Rectangle,
        state: &mut dyn std::any::Any,
    ) {
        if id == Some(&self.id) {
            let state: &mut State = state.downcast_mut().unwrap();
            state.should_center = true;
        }
    }
}

/// Create an [`Operation`] that will scale the image given an [`Id`].
pub fn scale_image_raw(id: widget::Id, scale: f32) -> impl Operation + 'static {
    ScaleImage {
        id: id.into(),
        scale,
    }
}

struct ScaleImage {
    id: widget::Id,
    scale: f32,
}

impl<T> Operation<T> for ScaleImage {
    fn container(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: iced_core::Rectangle,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
    ) {
        operate_on_children(self);
    }

    fn custom(
        &mut self,
        id: Option<&widget::Id>,
        _bounds: iced_core::Rectangle,
        state: &mut dyn std::any::Any,
    ) {
        if id == Some(&self.id) {
            let state: &mut State = state.downcast_mut().unwrap();
            state.suggested_scale = Some(self.scale);
        }
    }
}
