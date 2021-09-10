use wasm_bindgen::JsCast;

use crate::vdom::EventCallback;

use super::{component_manager::ComponentId, handle::AppHandle};

pub type EventCallbackClosure = wasm_bindgen::closure::Closure<dyn Fn(web_sys::Event)>;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct EventCallbackId(u32);

impl EventCallbackId {
    pub fn new_null() -> Self {
        Self(0)
    }

    fn is_null(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Clone)]
pub(crate) struct ComponentEventHandler {
    id: ComponentId,
    handler: EventCallback,
}

impl ComponentEventHandler {
    pub fn new(id: ComponentId, handler: EventCallback) -> Self {
        Self { id, handler }
    }

    #[inline]
    pub fn component_id(&self) -> ComponentId {
        self.id
    }

    pub fn handler(&self) -> &EventCallback {
        &self.handler
    }
}

// TODO: implement a swap and replace functionality to allow reusing the same
// handler without re-attaching by the patcher.
pub struct EventManager {
    // TODO: prevent need for option and unwraps. (?)
    app: Option<AppHandle>,
    /// Event handlers for the root app.
    handlers: Vec<ComponentEventHandler>,
    closures: Vec<EventCallbackClosure>,
    idle: Vec<usize>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            app: None,
            handlers: Vec::new(),
            closures: Vec::new(),
            idle: Vec::new(),
        }
    }

    pub(crate) fn set_app(&mut self, c: AppHandle) {
        self.app = Some(c);
    }

    pub(crate) fn get_handler(&self, id: EventCallbackId) -> Option<ComponentEventHandler> {
        self.handlers.get(id.0 as usize - 1).cloned()
    }

    pub(crate) fn get_closure_fn(&self, id: EventCallbackId) -> Option<&js_sys::Function> {
        self.closures
            .get(id.0 as usize - 1)
            .map(|x| x.as_ref().unchecked_ref())
    }

    // TODO: rename to build_callback
    pub(crate) fn build(
        &mut self,
        handler: ComponentEventHandler,
    ) -> (EventCallbackId, &js_sys::Function) {
        if let Some(index) = self.idle.pop() {
            // Old handler can be reused.
            self.handlers[index] = handler;
            (
                EventCallbackId((index as u32) + 1),
                self.closures[index].as_ref().unchecked_ref(),
            )
        } else {
            // Need to create a new one.
            let index = self.handlers.len();
            let id = EventCallbackId((index as u32) + 1);
            self.handlers.push(handler);
            {
                let id = id.clone();
                let app_handle = self
                    .app
                    // TODO: figure out why this needs to be an option...
                    .as_ref()
                    .expect("AppHandle not initialized")
                    .clone();
                let boxed: Box<dyn Fn(web_sys::Event)> = Box::new(move |event| {
                    app_handle.handle_event(id.clone(), event);
                });
                let closure = wasm_bindgen::closure::Closure::wrap(boxed);
                self.closures.push(closure);
            }
            (
                EventCallbackId((index as u32) + 1),
                self.closures[index].as_ref().unchecked_ref(),
            )
        }
    }

    pub(crate) fn recycle(&mut self, id: EventCallbackId) {
        if !id.is_null() && self.idle.len() < 100 {
            self.idle.push(id.0 as usize - 1);
        }
    }
}
