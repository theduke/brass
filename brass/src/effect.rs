use std::convert::TryInto;

use futures::{
    future::{AbortHandle, Abortable},
    Future,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;

use crate::web::window;

#[must_use]
pub struct EffectGuard {
    handle: AbortHandle,
}

impl Drop for EffectGuard {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

pub fn spawn_guarded<F: Future<Output = ()> + 'static>(f: F) -> EffectGuard {
    let (handle, reg) = AbortHandle::new_pair();
    let f = Abortable::new(f, reg);
    spawn_local(async move {
        f.await.ok();
    });
    EffectGuard { handle }
}

#[must_use]
pub struct TimeoutGuard {
    _closure: Closure<dyn FnMut()>,
    id: i32,
}

impl Drop for TimeoutGuard {
    fn drop(&mut self) {
        window().clear_timeout_with_handle(self.id);
    }
}

pub fn set_timeout(duration: std::time::Duration, f: impl FnOnce() + 'static) -> TimeoutGuard {
    // TODO: use callback cache
    let closure = Closure::once(move || {
        f();
    });

    let id = window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            duration.as_millis().try_into().unwrap(),
        )
        .unwrap();

    TimeoutGuard {
        _closure: closure,
        id,
    }
}

#[must_use]
pub struct IntervalGuard {
    _closure: Closure<dyn FnMut()>,
    id: i32,
}

impl Drop for IntervalGuard {
    fn drop(&mut self) {
        window().clear_interval_with_handle(self.id);
    }
}

pub fn set_interval(duration: std::time::Duration, mut f: impl FnMut() + 'static) -> IntervalGuard {
    // TODO: use callback cache
    let closure = Closure::wrap(Box::new(move || {
        f();
    }) as Box<dyn FnMut()>);

    let id = window()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            duration.as_millis().try_into().unwrap(),
        )
        .unwrap();

    IntervalGuard {
        _closure: closure,
        id,
    }
}

#[must_use]
pub struct EventSubscription {
    event: crate::dom::Event,
    target: web_sys::EventTarget,
    closure: Closure<dyn Fn(web_sys::Event)>,
}

impl EventSubscription {
    pub fn subscribe<E: wasm_bindgen::JsCast + 'static, F: Fn(E) + 'static>(
        target: web_sys::EventTarget,
        event: crate::dom::Event,
        callback: F,
    ) -> Self {
        // TODO: use callback cache
        let boxed: Box<dyn Fn(web_sys::Event)> = Box::new(move |event: web_sys::Event| {
            if let Ok(typed_ev) = event.dyn_into::<E>() {
                callback(typed_ev);
            }
        });
        let closure = wasm_bindgen::closure::Closure::wrap(boxed);

        target
            .add_event_listener_with_callback(event.as_str(), closure.as_ref().unchecked_ref())
            .unwrap();
        Self {
            event,
            target,
            closure,
        }
    }
}

impl Drop for EventSubscription {
    fn drop(&mut self) {
        if let Err(_err) = self.target.remove_event_listener_with_callback(
            self.event.as_str(),
            self.closure.as_ref().unchecked_ref(),
        ) {
            tracing::error!("Could not remove EventSubscription event listener");
        }
    }
}
