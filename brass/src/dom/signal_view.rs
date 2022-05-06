use futures::future::{AbortHandle, Abortable};
use futures_signals::signal::{Signal, SignalExt};

use crate::context::AppContext;

use super::{view::RetainedView, AbortGuard, View};

/// A view that is backed by a [`Signal`].
///
/// A future managing the signal runs on the render executor.
/// This proxy only stores the current rendered value to enable event handling
/// and cleanup.
pub struct SignalView(Box<Inner>);

struct Inner {
    current: RetainedView,
    _abort: AbortGuard,
    parent: Option<web_sys::Node>,
}

impl SignalView {
    pub(crate) fn replace_with(self, parent: &web_sys::Node, new_view: View) -> RetainedView {
        self.0.current.replace_with(parent, new_view)
    }

    pub(crate) fn replace(&self, parent: &web_sys::Node, old_node: &web_sys::Node) {
        let placeholder = self.0.current.as_placeholder().unwrap();
        parent.replace_child(placeholder, old_node).unwrap();
    }

    pub(crate) fn remove_from_parent(&self, parent: &web_sys::Node) {
        self.0.current.remove_from_parent(parent);
    }

    pub(crate) fn prepend_before_self(&self, parent: &web_sys::Node, new: &RetainedView) {
        self.0.current.prepend_before_self(parent, new);
    }

    pub(crate) fn insert_before(&self, parent: &web_sys::Node, before: &web_sys::Node) {
        self.0.current.insert_before(parent, before);
    }

    pub(crate) fn attach(&self, parent: &web_sys::Node) {
        parent
            .append_child(
                self.0
                    .current
                    .as_placeholder()
                    .expect("Can't attach a ViewSignal after it was initialized!"),
            )
            .unwrap();
    }

    pub(crate) fn new<T, S>(signal: S) -> Self
    where
        T: Into<View>,
        S: Signal<Item = T> + 'static,
    {
        let (handle, reg) = AbortHandle::new_pair();
        let mut inner = Box::new(Inner {
            current: RetainedView::new_placeholder(),
            _abort: AbortGuard::new(handle),
            parent: None,
        });

        let f = {
            // SAFETY:
            // Creates a 'static reference to the inner data.
            // The reference is only given to a future which is spawned on
            // a thread-local executor. The future is guarded by the [AbortGuard]
            // owned by the struct.
            // Since the futures executor is single-threaded and only runs on
            // the same thread the, future can't run after the struct has been
            // dropped (since it will be aborted) beforehand.
            let state = unsafe { &mut *(inner.as_mut() as *mut Inner) };

            signal.for_each(move |view| {
                let parent = if let Some(p) = &state.parent {
                    p
                } else if let Some(p) = state.current.as_placeholder().and_then(|p| p.parent_node())
                {
                    state.parent = Some(p);
                    state.parent.as_ref().unwrap()
                } else {
                    tracing::error!(
                        "ViewSignal received an update but is not attached to a parent!"
                    );
                    return std::future::ready(());
                };

                state.current.replace_with_mut(parent, view.into());
                std::future::ready(())
            })
        };

        let abortable_future = Abortable::new(f, reg);
        AppContext::spawn_custom_executor_unguarded(async move {
            abortable_future.await.ok();
        });

        Self(inner)
    }
}
