use std::{cell::RefCell, collections::VecDeque, marker::PhantomData, rc::Rc};
use wasm_bindgen::JsCast;

use web_sys::Node;

use crate::dom;

use super::{
    event_manager::{EventCallbackId, EventManager},
    Effect, EffectGuard, EventHandler, VNode,
};

enum ContextState<M> {
    Direct { effect: Rc<RefCell<Effect<M>>> },
    Mapped { register: Rc<dyn Fn(Effect<M>)> },
}

impl<M> Clone for ContextState<M> {
    fn clone(&self) -> Self {
        match self {
            ContextState::Direct { effect } => Self::Direct {
                effect: effect.clone(),
            },
            ContextState::Mapped { register } => Self::Mapped {
                register: register.clone(),
            },
        }
    }
}

pub struct Context<M> {
    state: ContextState<M>,
}

impl<M> Context<M> {
    fn new() -> Self {
        Self {
            state: ContextState::Direct {
                effect: Rc::new(RefCell::new(Effect::None)),
            },
        }
    }

    fn into_effect(self) -> Effect<M> {
        match self.state {
            ContextState::Direct { effect } => Rc::try_unwrap(effect)
                .ok()
                .expect("Context is still borroed")
                .into_inner(),
            ContextState::Mapped { register } => {
                #[allow(unreachable_code)]
                debug_assert!(
                    false,
                    panic!("Internal error: cant retreive effect for mapped request")
                );
                Effect::none()
            }
        }
    }

    fn add_effect(&self, eff: Effect<M>) {
        match &self.state {
            ContextState::Direct { effect } => {
                effect.borrow_mut().and(eff);
            }
            ContextState::Mapped { register } => {
                register(eff);
            }
        }
    }

    pub fn skip_render(&self) {
        self.add_effect(Effect::SkipRender);
    }

    pub fn run<F>(&self, f: F) -> EffectGuard
    where
        F: std::future::Future<Output = Option<M>> + 'static,
    {
        match &self.state {
            ContextState::Direct { effect } => {
                let (eff, guard) = Effect::future(f);
                effect.borrow_mut().and(eff);
                guard
            }
            ContextState::Mapped { register } => {
                let (eff, guard) = Effect::future(f);
                register(eff);
                guard
            }
        }
    }

    pub fn run_unguarded<F>(&self, f: F)
    where
        F: std::future::Future<Output = Option<M>> + 'static,
    {
        self.add_effect(Effect::future_unguarded(f));
    }

    pub(crate) fn duplicate(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }

    pub fn map<M2, F>(&self, mapper: F) -> Context<M2>
    where
        M: 'static,
        M2: 'static,
        F: Fn(M2) -> M + 'static + Clone,
    {
        let ctx = self.duplicate();
        Context {
            state: ContextState::Mapped {
                register: Rc::new(move |eff| {
                    // TODO: why is the clone neccessary?
                    ctx.add_effect(eff.map(mapper.clone()));
                }),
            },
        }
    }
}

pub trait Component: std::fmt::Debug + Sized + 'static {
    type Properties;
    type Msg: std::fmt::Debug + 'static;

    fn init(props: Self::Properties, context: &Context<Self::Msg>) -> Self;
    fn update(&mut self, msg: Self::Msg, context: &Context<Self::Msg>);
    fn render(&self) -> VNode<Self::Msg>;
}

pub(crate) struct ComponentState<C: Component> {
    window: web_sys::Window,
    document: web_sys::Document,
    component: C,
    parent: Node,
    vnode: VNode<C::Msg>,
    // Render callback scheduled with requestAnimationFrame.
    animation_callback: wasm_bindgen::closure::Closure<dyn FnMut()>,
    /// Flag to enable batching and prevent redundant renders.
    /// Set to true when a render has been scheduled with requestAnimationFrame,
    /// and set to false after a render.
    animation_pending: bool,

    // event_callback: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
    pub event_manager: EventManager<C>,
}

type SkipRender = bool;

impl<C: Component> ComponentState<C> {
    pub fn create_element(&self, tag: dom::Tag) -> web_sys::Element {
        self.document
            .create_element(tag.as_str())
            .expect("Could not create tag")
    }

    pub fn create_text_node(&self, text: &str) -> web_sys::Node {
        self.document.create_text_node(text).unchecked_into()
    }

    pub fn build_listener(
        &mut self,
        handler: EventHandler<C::Msg>,
    ) -> (EventCallbackId, &js_sys::Function) {
        self.event_manager.build(handler)
    }

    fn remove_listener(&mut self, id: EventCallbackId) {
        self.event_manager.recycle(id);
    }

    fn apply_effect(&mut self, effect: Effect<C::Msg>, handle: &ComponentHandle<C>) -> SkipRender {
        match effect {
            Effect::None => false,
            Effect::SkipRender => true,
            // Effect::Delay {
            //     msg,
            //     delay_until,
            //     guard,
            // } => false,
            Effect::Future { future, guard } => {
                let handle2 = handle.clone();
                // TODO: cancellation guard.
                wasm_bindgen_futures::spawn_local(async move {
                    let msg_opt = future.await;

                    match (msg_opt, guard) {
                        (Some(msg), Some(guard)) if !guard.is_cancelled() => {
                            handle2.update(msg);
                        }
                        (Some(msg), None) => {
                            handle2.update(msg);
                        }
                        _ => {
                            // Either no message, or cancelled.
                        }
                    }
                });
                false
            }
            Effect::Multi(items) => {
                let mut skip = false;
                for item in items {
                    if self.apply_effect(item, handle) {
                        skip = true;
                    }
                }
                skip
            }
        }
    }

    fn update(&mut self, msg: C::Msg, handle: &ComponentHandle<C>) {
        let context = Context::new();

        self.component.update(msg, &context);
        let effect = context.into_effect();
        self.apply_effect(effect, handle);

        if !self.animation_pending {
            self.animation_pending = true;
            // Note: error ignored for perf!
            self.window
                .request_animation_frame(self.animation_callback.as_ref().unchecked_ref())
                .ok();
        }
    }

    fn render(&mut self) {
        let mut new_vnode = self.component.render();
        let mut old_vnode = VNode::Empty;
        std::mem::swap(&mut old_vnode, &mut self.vnode);

        let parent = self.parent.clone();
        super::patch::patch(self, parent, None, old_vnode, &mut new_vnode);
        self.vnode = new_vnode;
        self.animation_pending = false;
    }
}

pub(crate) struct ComponentHandle<C: Component> {
    state: Rc<RefCell<ComponentState<C>>>,
    task_queue: Rc<RefCell<VecDeque<C::Msg>>>,
}

impl<C: Component> Clone for ComponentHandle<C> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            task_queue: self.task_queue.clone(),
        }
    }
}

impl<C: Component> ComponentHandle<C> {
    fn render(&self) {
        self.state.borrow_mut().render();
    }

    pub fn handle_event(&self, id: EventCallbackId, event: web_sys::Event) -> Option<()> {
        let mut s = self.state.borrow_mut();
        let handler = s.event_manager.get_handler(id)?;
        let msg = handler.invoke(event)?;
        s.update(msg, self);

        None
    }

    fn update(&self, msg: C::Msg) {
        self.task_queue.borrow_mut().push_back(msg);

        if let Ok(mut borrow) = self.state.try_borrow_mut() {
            while let Some(msg) = self.task_queue.borrow_mut().pop_front() {
                borrow.update(msg, self);
            }
        }
    }

    pub fn boot(props: C::Properties, parent: web_sys::Node) {
        let window = web_sys::window().expect("Could not retrieve window");
        let document = window.document().expect("Could not get document");

        let context = Context::new();
        let component = C::init(props, &context);
        let effect = context.into_effect();

        let state = Rc::new(RefCell::new(ComponentState {
            window,
            document,
            component,
            parent,
            vnode: VNode::Empty,
            animation_pending: false,
            // We first initialize the state with fake callbacks, since the real
            // ones need the Rc<> reference.
            animation_callback: wasm_bindgen::closure::Closure::wrap(Box::new(|| {})),
            event_manager: EventManager::new(),
        }));

        let s = Self {
            state,
            task_queue: Rc::new(RefCell::new(VecDeque::new())),
        };

        // Now we can replace the callbacks with real ones.
        {
            let handle1 = s.clone();
            let mut state = s.state.borrow_mut();
            state.animation_callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                handle1.render();
            }));

            state.event_manager.set_component(s.clone());

            state.render();

            state.apply_effect(effect, &s);
        }

        s.render();

        // TODO: figure out proper shutdown without leaking.
        std::mem::forget(s);
    }
}

pub trait ComponentMapper<P: Component, C: Component> {
    fn map_msg(msg: C::Msg) -> P::Msg;
    fn map_parent_msg(msg: P::Msg) -> Option<C::Msg>;
}

// #[derive(Debug)]
// pub struct ChildComponent<P, C> {
//     child: C,
//     marker: PhantomData<P>,
// }

// impl<P, C> Component for ChildComponent<P, C>
// where
//     P: Component,
//     C: Component,
//     C: ComponentMapper<P, C>,
// {
//     type Properties = C::Properties;
//     type Msg = P::Msg;

//     fn init(props: Self::Properties) -> (Self, Effect<Self::Msg>) {
//         let (child, eff) = C::init(props);
//         let s = Self {
//             child,
//             marker: PhantomData,
//         };
//         (s, eff.map(C::map_msg))
//     }

//     fn update(&mut self, msg: Self::Msg) -> Effect<Self::Msg> {
//         if let Some(child_msg) = C::map_parent_msg(msg) {
//             self.child.update(child_msg).map(C::map_msg)
//         } else {
//             Effect::none()
//         }
//     }

//     fn render(&self) -> VNode<Self::Msg> {
//         self.child.render().map(C::map_msg)
//     }
// }
