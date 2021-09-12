use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{closure::Closure, JsCast};

pub fn now() -> f64 {
    js_sys::Date::now()
}

pub fn window() -> web_sys::Window {
    // TODO: use OnceCell
    web_sys::window().expect("Could not get window")
}

pub fn document() -> web_sys::Document {
    // TODO: use OnceCell
    window().document().expect("Could not get document")
}

/// Get the current window URL pathname.
/// Equivalent to Javascript: `window.location.pathname`;
pub fn url_path() -> String {
    window().location().pathname().unwrap()
}

pub fn input_event_target(ev: web_sys::Event) -> Option<web_sys::HtmlInputElement> {
    ev.current_target()?
        .dyn_into::<web_sys::HtmlInputElement>()
        .ok()
}

pub fn input_event_value(ev: web_sys::Event) -> Option<String> {
    input_event_target(ev).map(|x| x.value())
}

pub fn input_event_checkbox_value(ev: web_sys::Event) -> Option<bool> {
    ev.current_target()?
        .dyn_ref::<web_sys::HtmlInputElement>()
        .map(|x| x.checked())
}

pub fn textarea_input_value(ev: web_sys::Event) -> Option<String> {
    let v = ev
        .current_target()?
        .dyn_ref::<web_sys::HtmlTextAreaElement>()?
        .value();
    Some(v)
}

pub fn query_selector(sel: &str) -> Option<web_sys::Element> {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector(sel)
        .ok()?
}

pub fn query_selector_as<T: wasm_bindgen::JsCast>(sel: &str) -> Option<T> {
    let elem = query_selector(sel)?;
    elem.dyn_into().ok()
}

/// A timeout future.
pub(crate) struct Timeout {
    closure: Option<Closure<dyn Fn()>>,
    // Javascript target time.
    target: f64,
    is_complete: Rc<RefCell<bool>>,
}

impl Timeout {
    fn new(target: f64) -> Self {
        Self {
            closure: None,
            target,
            is_complete: Rc::new(RefCell::new(false)),
        }
    }
}

impl std::future::Future for Timeout {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let s = self.get_mut();

        if s.closure.is_none() {
            let delay = (s.target - now()) as i32;
            if delay < 0 {
                // Already done.
                *s.is_complete.borrow_mut() = true;
                return std::task::Poll::Ready(());
            }

            // Schedule.
            let waker = cx.waker().to_owned();
            let flag = s.is_complete.clone();
            let c = Closure::wrap(Box::new(move || {
                *flag.borrow_mut() = true;
                waker.clone().wake();
            }) as Box<dyn Fn()>);
            window()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    c.as_ref().unchecked_ref(),
                    delay,
                )
                .expect("Could not set timeout");
            s.closure = Some(c);

            std::task::Poll::Pending
        } else {
            if *s.is_complete.borrow_mut() {
                std::task::Poll::Ready(())
            } else {
                std::task::Poll::Pending
            }
        }
    }
}

pub(crate) fn timeout(delay: std::time::Duration) -> Timeout {
    let target = now() + delay.as_millis() as f64;
    Timeout::new(target)
}

#[must_use]
pub struct EventSubscription {
    event: crate::dom::Event,
    target: web_sys::EventTarget,
    closure: Closure<dyn Fn(web_sys::Event)>,
}

impl EventSubscription {
    pub fn subscribe<E: wasm_bindgen::JsCast + 'static>(
        target: web_sys::EventTarget,
        event: crate::dom::Event,
        callback: crate::Callback<E>,
    ) -> Self {
        let boxed: Box<dyn Fn(web_sys::Event)> = Box::new(move |event: web_sys::Event| {
            if let Ok(typed_ev) = event.dyn_into::<E>() {
                callback.send(typed_ev);
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

    pub fn subscribe_filtered<E, M, F>(
        target: web_sys::EventTarget,
        event: crate::dom::Event,
        callback: crate::Callback<M>,
        filter: F,
    ) -> Self
    where
        E: wasm_bindgen::JsCast + 'static,
        F: Fn(E) -> Option<M> + 'static,
    {
        let boxed: Box<dyn Fn(web_sys::Event)> = Box::new(move |event: web_sys::Event| {
            if let Ok(typed_ev) = event.dyn_into::<E>() {
                if let Some(msg) = filter(typed_ev) {
                    callback.send(msg);
                }
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
        // FIXME: remove log
        tracing::trace!("REMOVING EVENT SUBSCRIPTION");
        if let Err(_err) = self.target.remove_event_listener_with_callback(
            self.event.as_str(),
            self.closure.as_ref().unchecked_ref(),
        ) {
            tracing::error!("Could not remove EventSubscription event listener");
        }
    }
}
