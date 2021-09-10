use std::marker::PhantomData;

use super::{EventCallback, EventHandler};
use crate::{any::any_box, app::EventCallbackId, Callback};

pub trait DomEvent: Sized {
    fn event_type() -> crate::dom::Event;
    fn from_dom(ev: web_sys::Event) -> Option<Self>;
}

impl DomEvent for web_sys::InputEvent {
    fn event_type() -> crate::dom::Event {
        crate::dom::Event::Input
    }

    fn from_dom(ev: web_sys::Event) -> Option<Self> {
        use wasm_bindgen::JsCast;
        ev.dyn_into().ok()
    }
}

// InputEvent

pub struct InputEvent(pub web_sys::InputEvent);

impl InputEvent {
    pub fn value(&self) -> Option<String> {
        use wasm_bindgen::JsCast;

        let target = self.0.current_target()?;

        // TODO perf: can these 3 checks be reduced to just one?
        if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
            Some(input.value())
        } else if let Some(textarea) = target.dyn_ref::<web_sys::HtmlTextAreaElement>() {
            Some(textarea.value())
        } else if let Some(select) = target.dyn_ref::<web_sys::HtmlSelectElement>() {
            Some(select.value())
        } else {
            None
        }
    }
}

impl std::ops::Deref for InputEvent {
    type Target = web_sys::InputEvent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DomEvent for InputEvent {
    fn event_type() -> crate::dom::Event {
        crate::dom::Event::Input
    }

    fn from_dom(ev: web_sys::Event) -> Option<Self> {
        use wasm_bindgen::JsCast;
        ev.dyn_into().ok().map(Self)
    }
}

// ChangeEvent

pub struct ChangeEvent(pub web_sys::Event);

impl ChangeEvent {
    pub fn value(&self) -> Option<String> {
        use wasm_bindgen::JsCast;

        let target = self.0.current_target()?;

        // TODO perf: can these 3 checks be reduced to just one?
        if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
            Some(input.value())
        } else if let Some(textarea) = target.dyn_ref::<web_sys::HtmlTextAreaElement>() {
            Some(textarea.value())
        } else if let Some(select) = target.dyn_ref::<web_sys::HtmlSelectElement>() {
            Some(select.value())
        } else {
            None
        }
    }
}

impl std::ops::Deref for ChangeEvent {
    type Target = web_sys::Event;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DomEvent for ChangeEvent {
    fn event_type() -> crate::dom::Event {
        crate::dom::Event::Change
    }

    fn from_dom(ev: web_sys::Event) -> Option<Self> {
        use wasm_bindgen::JsCast;
        ev.dyn_into().ok().map(Self)
    }
}

// ClickEvent.

pub struct ClickEvent(pub web_sys::MouseEvent);

impl std::ops::Deref for ClickEvent {
    type Target = web_sys::MouseEvent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DomEvent for ClickEvent {
    fn event_type() -> crate::dom::Event {
        crate::dom::Event::Click
    }

    fn from_dom(ev: web_sys::Event) -> Option<Self> {
        use wasm_bindgen::JsCast;
        ev.dyn_into().ok().map(Self)
    }
}

// KeyDownEvent.

pub struct KeyDownEvent(pub web_sys::KeyboardEvent);

impl std::ops::Deref for KeyDownEvent {
    type Target = web_sys::KeyboardEvent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DomEvent for KeyDownEvent {
    fn event_type() -> crate::dom::Event {
        crate::dom::Event::KeyDown
    }

    fn from_dom(ev: web_sys::Event) -> Option<Self> {
        use wasm_bindgen::JsCast;
        ev.dyn_into().ok().map(Self)
    }
}

pub trait EventHandlerFn<E, M, X> {
    fn into_handler(self) -> EventHandler;
    fn into_handler_callback(self, cb: &Callback<M>) -> EventHandler;
}

impl<E, M, F> EventHandlerFn<E, M, PhantomData<&M>> for F
where
    E: DomEvent,
    M: 'static,
    F: Fn(E) -> M + 'static,
{
    fn into_handler(self) -> EventHandler {
        let cb = EventCallback::Closure(std::rc::Rc::new(move |ev: web_sys::Event| {
            if let Some(mapped_event) = E::from_dom(ev) {
                Some(any_box(self(mapped_event)))
            } else {
                None
            }
        }));
        EventHandler::new(E::event_type(), cb)
    }

    fn into_handler_callback(self, cb: &Callback<M>) -> EventHandler {
        let cb = if let Some(mapper) = cb.mapper() {
            let callback_mapper = mapper.clone();

            let f = std::rc::Rc::new(move |ev: web_sys::Event| {
                if let Some(mapped_event) = E::from_dom(ev) {
                    let mapped_data = self(mapped_event);
                    let callback_mapped_data = (callback_mapper.as_ref())(mapped_data);
                    Some(callback_mapped_data)
                } else {
                    None
                }
            });

            EventCallback::ComponentCallback {
                component_id: cb.component_id(),
                handler: f,
            }
        } else {
            let f = std::rc::Rc::new(move |ev: web_sys::Event| {
                if let Some(mapped_event) = E::from_dom(ev) {
                    Some(any_box(self(mapped_event)))
                } else {
                    None
                }
            });
            EventCallback::ComponentCallback {
                component_id: cb.component_id(),
                handler: f,
            }
        };

        EventHandler {
            event: E::event_type(),
            callback: cb,
            callback_id: EventCallbackId::new_null(),
        }
    }
}

impl<E, M, F> EventHandlerFn<E, M, PhantomData<&Option<M>>> for F
where
    E: DomEvent,
    M: 'static,
    F: Fn(E) -> Option<M> + 'static,
{
    fn into_handler(self) -> EventHandler {
        let cb = EventCallback::Closure(std::rc::Rc::new(move |ev: web_sys::Event| {
            if let Some(mapped_event) = E::from_dom(ev) {
                self(mapped_event).map(any_box)
            } else {
                None
            }
        }));
        EventHandler::new(E::event_type(), cb)
    }

    fn into_handler_callback(self, cb: &Callback<M>) -> EventHandler {
        let cb = if let Some(mapper) = cb.mapper() {
            let callback_mapper = mapper.clone();

            let f = std::rc::Rc::new(move |ev: web_sys::Event| {
                if let Some(mapped_event) = E::from_dom(ev) {
                    if let Some(mapped_data) = self(mapped_event) {
                        let callback_mapped_data = (callback_mapper.as_ref())(mapped_data);
                        Some(callback_mapped_data)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

            EventCallback::ComponentCallback {
                component_id: cb.component_id(),
                handler: f,
            }
        } else {
            let f = std::rc::Rc::new(move |ev: web_sys::Event| {
                if let Some(mapped_event) = E::from_dom(ev) {
                    Some(any_box(self(mapped_event)))
                } else {
                    None
                }
            });
            EventCallback::ComponentCallback {
                component_id: cb.component_id(),
                handler: f,
            }
        };

        EventHandler {
            event: E::event_type(),
            callback: cb,
            callback_id: EventCallbackId::new_null(),
        }
    }
}
