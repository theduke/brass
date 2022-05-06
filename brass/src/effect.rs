use futures::Future;
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::{
    context::{AppContext, EventHandlerRef},
    dom::AbortGuard,
    web::window,
};

#[must_use]
pub struct EffectGuard {
    _handle: AbortGuard,
}

pub fn spawn_guarded<F: Future<Output = ()> + 'static>(f: F) -> EffectGuard {
    let guard = AppContext::spawn_external_abortable(f);

    EffectGuard { _handle: guard }
}

#[must_use]
pub struct TimeoutGuard {
    pub(crate) _closure: Closure<dyn FnMut()>,
    pub(crate) id: i32,
}

impl Drop for TimeoutGuard {
    fn drop(&mut self) {
        window().clear_timeout_with_handle(self.id);
    }
}

pub fn set_timeout(duration: std::time::Duration, f: impl FnOnce() + 'static) -> TimeoutGuard {
    AppContext::create_timeout(duration, f)
}

pub struct TimeoutFuture {
    _guard: TimeoutGuard,
    receiver: futures::channel::oneshot::Receiver<()>,
}

impl TimeoutFuture {
    pub fn new(duration: std::time::Duration) -> Self {
        let (tx, receiver) = futures::channel::oneshot::channel();

        let _guard = set_timeout(duration, move || {
            tx.send(()).ok();
        });

        Self { _guard, receiver }
    }
}

impl std::future::Future for TimeoutFuture {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::pin::Pin::new(&mut self.receiver).poll(cx).map(|_| ())
    }
}

#[must_use]
pub struct IntervalGuard {
    pub(crate) _closure: Closure<dyn FnMut()>,
    pub(crate) id: i32,
}

impl Drop for IntervalGuard {
    fn drop(&mut self) {
        window().clear_interval_with_handle(self.id);
    }
}

pub fn set_interval(duration: std::time::Duration, f: impl FnMut() + 'static) -> IntervalGuard {
    AppContext::create_interval(duration, f)
}

#[must_use]
pub struct EventSubscription(EventHandlerRef);

impl EventSubscription {
    pub fn subscribe<E: wasm_bindgen::JsCast + 'static, F: Fn(E) + 'static>(
        target: web_sys::EventTarget,
        event: crate::dom::Ev,
        callback: F,
    ) -> Self {
        let wrapped = move |event: web_sys::Event| {
            if let Ok(typed_ev) = event.dyn_into::<E>() {
                callback(typed_ev);
            }
        };

        let r = AppContext::create_event_listener(event, wrapped, target);
        Self(r)
    }
}
