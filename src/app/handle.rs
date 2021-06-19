use std::{cell::RefCell, rc::Rc};

use crate::any::AnyBox;

use super::{
    component::Component, component_manager::ComponentId, event_manager::EventCallbackId,
    state::AppState,
};

pub(crate) enum Task {
    Message {
        component_id: ComponentId,
        msg: AnyBox,
    },
    Event {
        callback_id: EventCallbackId,
        event: web_sys::Event,
    },
}

#[derive(Clone)]
pub(crate) struct AppHandle {
    inner: Rc<(RefCell<AppState>, RefCell<Vec<Task>>)>,
}

impl AppHandle {
    fn borrow_state_mut(&self) -> std::cell::RefMut<AppState> {
        self.inner.0.borrow_mut()
    }

    fn borrow_queue_mut(&self) -> std::cell::RefMut<Vec<Task>> {
        self.inner.1.borrow_mut()
    }

    fn process_tasks(&self, state: &mut AppState) {
        loop {
            let tasks = { std::mem::take(&mut *self.borrow_queue_mut()) };
            if tasks.is_empty() {
                break;
            }
            for task in tasks {
                match task {
                    Task::Event { callback_id, event } => {
                        state.handle_event(callback_id, event);
                    }
                    Task::Message { component_id, msg } => {
                        state.update_component(component_id, msg);
                    }
                }
            }
        }
    }

    pub fn handle_event(&self, callback_id: EventCallbackId, event: web_sys::Event) {
        self.update(Task::Event { callback_id, event });
    }

    pub(crate) fn update(&self, task: Task) {
        {
            self.borrow_queue_mut().push(task)
        }
        if let Ok(mut state) = self.inner.0.try_borrow_mut() {
            self.process_tasks(&mut state);
        }
    }

    pub fn send_message(&self, component_id: ComponentId, msg: AnyBox) {
        self.update(Task::Message { component_id, msg });
    }

    pub fn render(&self) {
        self.borrow_state_mut().render();
    }

    pub fn boot<C: Component>(props: C::Properties, parent: web_sys::Element) {
        // FIXME: use proper handle  instead of fake one.
        // let handle = ComponentAppHandle {
        //     component_id: ComponentId::NONE,
        //     app: Rc::new(|m| {
        //         todo!("tried to use fake component handle");
        //     }),
        // };

        let state = RefCell::new(AppState::new());

        let queue = RefCell::new(Vec::new());
        let handle = Self {
            inner: Rc::new((state, queue)),
        };

        handle
            .borrow_state_mut()
            .boot::<C>(handle.clone(), props, parent);

        // TODO: figure out proper shutdown without leaking.
        std::mem::forget(handle);
    }
}

#[derive(Clone)]
pub struct ComponentAppHandle {
    component_id: ComponentId,
    app: AppHandle,
}

impl ComponentAppHandle {
    pub(crate) fn new(component_id: ComponentId, app: AppHandle) -> Self {
        Self { component_id, app }
    }

    pub fn send_message(&self, msg: AnyBox) {
        self.app.send_message(self.component_id, msg);
    }
}
