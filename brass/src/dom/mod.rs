mod attribute;
mod event;
mod node;
mod signal_vec_view;
mod signal_view;
mod style;
mod tag;
mod view;

pub use self::{
    attribute::Attr,
    event::{ChangeEvent, CheckboxInputEvent, ClickEvent, DomEvent, Ev, InputEvent, KeyDownEvent},
    node::{
        builder, Apply, ApplyFuture, AttrValueApply, EventHandlerApply, Fragment, Node, Render,
        TagBuilder, WithSignal,
    },
    style::Style,
    tag::Tag,
    view::View,
};

/// A guard for a future [`AbortHandle`].
/// If dropped, the related future will be aborted.
pub struct AbortGuard(pub(crate) futures::future::AbortHandle);

impl AbortGuard {
    pub fn new(handle: futures::future::AbortHandle) -> Self {
        Self(handle)
    }
}

impl Drop for AbortGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}
