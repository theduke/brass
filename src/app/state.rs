use std::collections::HashMap;

use wasm_bindgen::JsCast;

use crate::{any::AnyBox, vdom::VComponent, Component};

use super::{
    component::ComponentConstructor,
    component_manager::{ComponentId, ComponentManager},
    event_manager::{EventCallbackId, EventManager},
    handle::{AppHandle, ComponentAppHandle},
};

pub struct ContextContainer {
    values: HashMap<std::any::TypeId, AnyBox>,
}

pub(crate) struct Timer {
    perf: once_cell::unsync::Lazy<web_sys::Performance>,
    last_timestamp: f64,
    render: f64,
    patch: f64,
}

impl Timer {
    #[inline]
    pub fn finish_render() {
        #[cfg(feature = "timings")]
        unsafe {
            let now = TIMER.perf.now();
            TIMER.render += now - TIMER.last_timestamp;
            TIMER.last_timestamp = now;
        }
    }

    #[inline]
    pub fn finish_patch() {
        #[cfg(feature = "timings")]
        unsafe {
            TIMER.patch += 0.0;
            let now = TIMER.perf.now();
            TIMER.render += now - TIMER.last_timestamp;
            TIMER.last_timestamp = now;
        }
    }

    #[inline]
    pub fn start_rendering() {
        #[cfg(feature = "timings")]
        unsafe {
            TIMER.last_timestamp = TIMER.perf.now();
            TIMER.render = 0.0;
            TIMER.patch = 0.0;
        }
    }

    #[inline]
    pub fn finish_rendering() {
        #[cfg(feature = "timings")]
        unsafe {
            tracing::trace!(time_render=%TIMER.render, time_patch=%TIMER.patch, "rendered");
        }
    }
}

pub(crate) static mut TIMER: Timer = Timer {
    perf: once_cell::unsync::Lazy::new(|| web_sys::window().unwrap().performance().unwrap()),
    last_timestamp: 0.0,
    render: 0.0,
    patch: 0.0,
};

impl ContextContainer {
    pub fn new() -> Self {
        Self {
            values: Default::default(),
        }
    }

    pub fn register<T: 'static>(&mut self, value: T) {
        let id = std::any::TypeId::of::<T>();
        self.values.insert(id, Box::new(value));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        let id = std::any::TypeId::of::<T>();
        let value = self.values.get(&id)?;
        value.downcast_ref()
    }

    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        let id = std::any::TypeId::of::<T>();
        let value = self.values.remove(&id)?;
        value.downcast::<T>().ok().map(|x| *x)
    }
}

pub(crate) struct AppState {
    pub window: web_sys::Window,
    pub document: web_sys::Document,
    // Render callback scheduled with requestAnimationFrame.
    animation_callback: wasm_bindgen::closure::Closure<dyn FnMut()>,

    pub event_manager: EventManager,
    component_manager: ComponentManager,
    pub context: ContextContainer,

    root_render_queued: bool,
    render_queue: Vec<ComponentId>,

    // TODO: try to prevent the option.
    handle: Option<AppHandle>,
}

impl AppState {
    pub fn make_component_handle(&self, component_id: ComponentId) -> ComponentAppHandle {
        ComponentAppHandle::new(component_id, self.handle.clone().unwrap())
    }

    #[inline]
    pub fn component_manager(&mut self) -> &mut ComponentManager {
        &mut self.component_manager
    }

    fn mount_component(
        &mut self,
        constructor: &ComponentConstructor,
        props: AnyBox,
        parent: &web_sys::Element,
        next_sibling: Option<&web_sys::Node>,
    ) -> (ComponentId, Option<web_sys::Node>) {
        let finisher = self.component_manager.reserve_id();
        let id = finisher.id();

        let state = constructor.call(self, id, props, parent.clone(), next_sibling.cloned());
        let node = state.node().cloned();
        finisher.return_component(&mut self.component_manager, Box::new(state));
        (id, node)
    }

    pub fn mount_virtual_component<'a, 'b, 'c>(
        &'a mut self,
        vcomp: &'b mut VComponent,
        parent: &web_sys::Element,
        next_sibling: Option<&web_sys::Node>,
    ) -> Option<web_sys::Node> {
        if vcomp.id.is_none() {
            // New component.

            let (id, node) = self.mount_component(
                &vcomp.spec.constructor,
                vcomp.spec.props.take().expect("No properties set"),
                parent,
                next_sibling,
            );
            vcomp.id = id;
            node
        } else {
            // Existing component.

            let (mut comp, finisher) = self
                .component_manager
                .borrow(vcomp.id)
                .expect("Component has gone away");

            let node = comp.remount(
                self,
                vcomp.spec.props.take().unwrap_or_else(|| Box::new(())),
            );

            finisher.return_component(&mut self.component_manager, comp);

            node
        }
    }

    /// Schedule a re-render via requestAnimationFrame.
    /// If no child component id is given, the update is for the root.
    fn schedule_render_if_needed(&mut self, component: Option<ComponentId>) {
        let needs_schedule = self.render_queue.is_empty() && !self.root_render_queued;

        if let Some(id) = component {
            self.render_queue.push(id);
        } else {
            self.root_render_queued = true;
        }

        if needs_schedule {
            self.window
                .request_animation_frame(self.animation_callback.as_ref().unchecked_ref())
                .ok();
        }
    }

    pub fn update_component(&mut self, component_id: ComponentId, msg: AnyBox) {
        let (mut comp, finisher) = self
            .component_manager
            .borrow(component_id)
            .expect(&format!("Component disappeared: {:?}", component_id));

        let should_render = comp.update(self, msg);
        if should_render {
            self.schedule_render_if_needed(Some(component_id));
        }

        finisher.return_component(&mut self.component_manager, comp);
    }

    fn render_component(&mut self, component_id: ComponentId) {
        let opt = self.component_manager.borrow(component_id);

        let (mut comp, finisher) = match opt {
            Some((c, f)) => (c, f),
            None => {
                tracing::error!("Component has disappeared");
                return;
            }
        };

        comp.render(self);
        finisher.return_component(&mut self.component_manager, comp);
    }

    pub fn render(&mut self) {
        Timer::start_rendering();

        // FIXME: determine the minimal sub-tree to re-render.
        while let Some(id) = self.render_queue.pop() {
            self.render_component(id);
        }
        self.render_component(ComponentId::ROOT);

        Timer::finish_rendering();
    }

    pub fn handle_event(
        &mut self,
        callback_id: EventCallbackId,
        event: web_sys::Event,
    ) -> Option<()> {
        let handler = self.event_manager.get_handler(callback_id)?;
        let msg = handler.invoke(event)?;
        self.update_component(handler.component_id(), msg);
        None
    }

    /// Build a new AppState.
    /// [`Self::boot`] must be called to actually start the app.
    pub fn new() -> Self {
        let window = web_sys::window().expect("Could not retrieve window");
        let document = window.document().expect("Could not get document");

        Self {
            window,
            document,
            root_render_queued: false,
            render_queue: Vec::new(),

            event_manager: EventManager::new(),
            component_manager: ComponentManager::new(),
            context: ContextContainer::new(),

            // We first initialize the state with fake callbacks, since the real
            // ones need the AppHandle reference.
            // Properly initialized in Self::boot.
            handle: None,
            animation_callback: wasm_bindgen::closure::Closure::wrap(Box::new(|| {})),
        }
    }

    pub fn boot<C: Component>(
        &mut self,
        handle: AppHandle,
        props: C::Properties,
        parent: web_sys::Element,
    ) {
        self.handle = Some(handle.clone());
        let handle1 = handle.clone();

        self.animation_callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            handle1.render();
        }));

        self.event_manager.set_app(handle.clone());

        let cons = ComponentConstructor::new::<C>();
        self.mount_component(&cons, Box::new(props), &parent, None);
    }
}
