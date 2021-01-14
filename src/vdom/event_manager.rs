use wasm_bindgen::JsCast;

use super::{EventHandler, component::{Component, ComponentHandle}};


pub type EventCallbackClosure = wasm_bindgen::closure::Closure<dyn Fn(web_sys::Event)>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct EventCallbackId(u32);

impl EventCallbackId {
    pub fn new_null() -> Self {
        Self(0)
    }

    fn is_null(&self) -> bool {
        self.0 == 0
    }
}

// TODO: implement a swap and replace functionality to allow reusing the same
// handler without re-attaching by the patcher.
pub struct EventManager<C: Component> {
    // TODO: prevent need for option and unwraps. (?)
    component: Option<ComponentHandle<C>>,
    handlers: Vec<EventHandler<C::Msg>>,
    closures: Vec<EventCallbackClosure>,
    idle: Vec<usize>,
}

impl<C: Component> EventManager<C> {
    pub fn new() -> Self {
        Self {
            component: None,
            handlers: Vec::new(),
            closures: Vec::new(),
            idle: Vec::new(),
        }
    }

    pub(crate) fn set_component(&mut self, c: ComponentHandle<C>) {
        self.component = Some(c);
    }

    pub(crate) fn get_handler(&self, id: EventCallbackId) -> Option<EventHandler<C::Msg>> {
        self.handlers.get(id.0 as usize - 1).cloned()
    }

    pub(crate) fn get_closure_fn(&self, id: EventCallbackId) -> Option<&js_sys::Function> {
        self.closures
            .get(id.0 as usize - 1)
            .map(|x| x.as_ref().unchecked_ref())
    }

    pub(crate) fn build(&mut self, handler: EventHandler<C::Msg>) -> (EventCallbackId, &js_sys::Function) {
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
                let component = self
                    .component
                    .as_ref()
                    .expect("Uninitialized component in EventManager")
                    .clone();
                let boxed: Box<dyn Fn(web_sys::Event)> = Box::new(move |event| {
                    component.handle_event(id.clone(), event);
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
        if !id.is_null() {
            self.idle.push(id.0 as usize - 1);
        }
    }
}
