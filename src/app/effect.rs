use std::{cell::Cell, pin::Pin, rc::Rc};

use crate::{any::AnyBox, vdom::EventCallback};

use super::handle::ComponentAppHandle;

#[derive(Debug)]
#[must_use]
pub struct EffectGuard {
    is_cancelled: Rc<Cell<bool>>,
}

impl EffectGuard {
    /// Manually cancel the effect.
    /// Note that cancellation happens automatically when the guard is dropped,
    /// so this method is rarely useful.
    pub fn cancel(self) {}

    pub(crate) fn new() -> (Self, EffectGuardHandle) {
        let is_cancelled = Rc::new(Cell::new(false));
        let handle = EffectGuardHandle {
            is_cancelled: is_cancelled.clone(),
        };
        let s = Self { is_cancelled };
        (s, handle)
    }
}

impl Drop for EffectGuard {
    fn drop(&mut self) {
        self.is_cancelled.set(true);
    }
}

/// A handle that allows checking if an [`EffectGuard`] has been dropped.
#[derive(Clone)]
pub(crate) struct EffectGuardHandle {
    is_cancelled: Rc<Cell<bool>>,
}

impl EffectGuardHandle {
    /// Check if the effect was cancelled.
    /// Note: also returns true if the future has completed.
    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.get()
    }

    // pub fn force_cancel(&self) {
    //     self.is_cancelled.set(true);
    // }
}

/// A future that supports cancellation.
pub(crate) struct EffectFuture<F> {
    app_handle: ComponentAppHandle,
    guard_handle: EffectGuardHandle,
    inner: Pin<Box<F>>,
}

impl<F> EffectFuture<F>
where
    F: std::future::Future<Output = Option<AnyBox>> + Unpin,
{
    pub(super) fn new(app: ComponentAppHandle, f: F) -> (EffectFuture<F>, EffectGuard) {
        let (guard, handle) = EffectGuard::new();

        (
            Self {
                app_handle: app,
                guard_handle: handle,
                inner: Pin::new(Box::new(f)),
            },
            guard,
        )
    }
}

impl<F> std::future::Future for EffectFuture<F>
where
    F: std::future::Future<Output = Option<AnyBox>> + Unpin,
{
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let s = self.get_mut();
        if !s.guard_handle.is_cancelled() {
            match s.inner.as_mut().poll(cx) {
                std::task::Poll::Ready(msg_opt) => {
                    // Set the cancelled flag to true.
                    // This both makes the future fused, and allows EffectGuardHandle
                    // owners to check if the handle is still useful.
                    // TODO: this is probably redundant?
                    s.guard_handle.is_cancelled.set(true);
                    if let Some(msg) = msg_opt {
                        s.app_handle.send_message(msg);
                    }
                    std::task::Poll::Ready(())
                }
                std::task::Poll::Pending => std::task::Poll::Pending,
            }
        } else {
            std::task::Poll::Ready(())
        }
    }
}

type AnyMapper<T> = Rc<dyn Fn(T) -> AnyBox>;

/// A callback that allows sending messages to a component.
pub struct Callback<M: 'static> {
    handle: ComponentAppHandle,
    mapper: Option<AnyMapper<M>>,
}

impl<M: 'static> Clone for Callback<M> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            mapper: self.mapper.clone(),
        }
    }
}

impl<M: 'static> Callback<M> {
    pub(crate) fn new(handle: ComponentAppHandle) -> Self {
        Self {
            handle,
            mapper: None,
        }
    }

    pub fn send(&self, value: M) {
        if let Some(mapper) = &self.mapper {
            self.handle.send_message(mapper(value))
        } else {
            self.handle.send_message(Box::new(value))
        }
    }

    pub fn map<T: 'static, F: Fn(T) -> M + 'static>(self, mapper: F) -> Callback<T> {
        let nested_mapper = match self.mapper {
            Some(old_mapper) => Rc::new(move |value: T| -> AnyBox {
                let first_mapped = mapper(value);
                old_mapper(first_mapped)
            }) as AnyMapper<T>,
            None => Rc::new(move |value: T| -> AnyBox { Box::new(mapper(value)) }),
        };

        Callback {
            handle: self.handle.clone(),
            mapper: Some(nested_mapper),
        }
    }

    /// Construct an event handler that triggers this callback.
    ///
    /// Must supply a mapper that transforms the DOM event into the expected
    /// message.
    pub fn on<F>(self, mapper: F) -> EventCallback
    where
        F: Fn(web_sys::Event) -> M + 'static,
    {
        EventCallback::callback(mapper, self)
    }

    /// Construct an event handler that triggers this callback.
    ///
    /// Must supply a mapper that produces the expected message.
    pub fn on_simple<F>(self, f: F) -> EventCallback
    where
        F: Fn() -> M + 'static,
    {
        EventCallback::callback(move |_ev: web_sys::Event| f(), self)
    }
}
