use futures::{FutureExt, StreamExt};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};

use web_sys::Node;

use crate::dom;

use super::{
    event_manager::{EventCallbackId, EventManager, ManagedEvent},
    patch::patch,
    Effect, EffectGuard, EventHandler, VComponent, VNode,
};

pub type AnyBox = Box<dyn std::any::Any>;
pub fn into_any_box(value: impl std::any::Any) -> AnyBox {
    Box::new(value)
}

enum ContextState<'a, M> {
    Direct { effect: &'a mut Effect<M> },
    Nested { effect: &'a mut Effect<AnyBox> },
}

pub struct Context<'a, M> {
    state: ContextState<'a, M>,
}

impl<'a> Context<'a, AnyBox> {
    fn into_nested<M2>(self) -> Context<'a, M2> {
        match self.state {
            ContextState::Direct { effect } => Context {
                state: ContextState::Nested { effect },
            },
            ContextState::Nested { effect } => Context {
                state: ContextState::Nested { effect },
            },
        }
    }
}

impl<'a, M> Context<'a, M> {
    fn new_direct(effect: &'a mut Effect<M>) -> Self {
        Self {
            state: ContextState::Direct { effect },
        }
    }

    pub fn skip_render(&mut self) {
        match &mut self.state {
            ContextState::Direct { effect } => effect.and(Effect::SkipRender),
            ContextState::Nested { effect } => effect.and(Effect::SkipRender),
        }
    }

    pub fn navigate(&mut self, route: String) {
        match &mut self.state {
            ContextState::Direct { effect } => effect.and(Effect::Navigate(route)),
            ContextState::Nested { effect } => effect.and(Effect::Navigate(route)),
        }
    }

    pub fn run<F>(&mut self, f: F) -> EffectGuard
    where
        M: 'static,
        F: std::future::Future<Output = Option<M>> + 'static,
    {
        match &mut self.state {
            ContextState::Direct { effect } => {
                let (eff, guard) = Effect::future(f);
                effect.and(eff);
                guard
            }
            ContextState::Nested { effect } => {
                let (eff, guard) = Effect::future(f.map(|opt| opt.map(into_any_box)));
                effect.and(eff);
                guard
            }
        }
    }

    pub fn run_unguarded<F>(&mut self, f: F)
    where
        M: 'static,
        F: std::future::Future<Output = Option<M>> + 'static,
    {
        match &mut self.state {
            ContextState::Direct { effect } => {
                effect.and(Effect::future_unguarded(f));
            }
            ContextState::Nested { effect } => {
                let eff = Effect::future_unguarded(f.map(|opt| opt.map(into_any_box)));
                effect.and(eff);
            }
        }
    }

    pub fn subscribe<S>(&mut self, stream: S) -> EffectGuard
    where
        M: 'static,
        S: futures::stream::Stream<Item = M> + 'static,
    {
        match &mut self.state {
            ContextState::Direct { effect } => {
                let (eff, guard) = Effect::subscribe(stream);
                effect.and(eff);
                guard
            }
            ContextState::Nested { effect } => {
                let mapped_stream = stream.map(into_any_box);
                let (eff, guard) = Effect::subscribe(mapped_stream);
                effect.and(eff);
                guard
            }
        }
    }

    pub fn subscribe_forever<S>(&mut self, stream: S)
    where
        M: 'static,
        S: futures::stream::Stream<Item = M> + 'static,
    {
        match &mut self.state {
            ContextState::Direct { effect } => {
                effect.and(Effect::subscribe_unguarded(stream));
            }
            ContextState::Nested { effect } => {
                let mapped_stream = stream.map(into_any_box);
                effect.and(Effect::subscribe_unguarded(mapped_stream));
            }
        }
    }
}

pub trait Component: std::fmt::Debug + Sized + 'static {
    type Properties;
    type Msg: std::fmt::Debug + 'static;

    fn init(props: Self::Properties, context: Context<Self::Msg>) -> Self;
    fn on_property_change(&mut self, new_props: Self::Properties, context: Context<Self::Msg>);
    fn update(&mut self, msg: Self::Msg, context: Context<Self::Msg>);
    fn render(&self) -> VNode<Self::Msg>;
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct ComponentId(u32);

impl ComponentId {
    pub(crate) fn new_none() -> Self {
        Self(0)
    }
}

type BoxedComponent = Box<dyn DynamicComponent>;

/// A wrapper around a function pointer that constructs a boxed component.
/// Used in [`VComponent`] for describing the component and by App to create it.
struct ComponentConstructor(fn(AnyBox, Context<AnyBox>) -> (BoxedComponent, VNode<AnyBox>));

impl ComponentConstructor {
    #[inline]
    fn call(&self, props: AnyBox, context: Context<AnyBox>) -> (BoxedComponent, VNode<AnyBox>) {
        (self.0)(props, context)
    }
}

trait DynamicComponent {
    fn on_property_change(
        &mut self,
        new_props: AnyBox,
        context: Context<AnyBox>,
    ) -> Option<VNode<AnyBox>>;
    fn update(&mut self, msg: AnyBox, ctx: Context<AnyBox>);
    fn render(&self) -> VNode<AnyBox>;
}

impl<C: Component> DynamicComponent for C {
    fn on_property_change(
        &mut self,
        new_props: AnyBox,
        ctx: Context<AnyBox>,
    ) -> Option<VNode<AnyBox>> {
        let real_props = *(new_props
            .downcast::<C::Properties>()
            .expect("Invalid property type"));
        let real_context = ctx.into_nested();
        self.on_property_change(real_props, real_context);

        // FIXME: determine if we should render.
        // let node = c.render();
        // node.map(into_any_box)
        None
    }

    fn update(&mut self, msg: AnyBox, ctx: Context<AnyBox>) {
        let real_msg = *msg
            .downcast::<C::Msg>()
            .expect("Internal error: invalid message type");
        let real_context = ctx.into_nested();
        self.update(real_msg, real_context);
    }

    fn render(&self) -> VNode<AnyBox> {
        // TODO: can we map with less overhead?
        self.render().map(into_any_box)
    }
}

struct ComponentState2<T> {
    /// The boxed component.
    component: T,
    /// VNode from the previous render.
    last_vnode: VNode<AnyBox>,
    /// Parent dom element.
    parent: web_sys::Element,
    /// Next sibling. Needed for dom patching.
    next_sibling: Option<web_sys::Node>,
    node: Option<web_sys::Node>,
}

type DynamicComponentState = ComponentState2<Box<dyn DynamicComponent>>;

pub(crate) struct ComponentSpec {
    constructor: ComponentConstructor,
    // TODO: use Option<>  for components without properties to avoid allocations.
    // Properties for the component.
    // Will be used during rendering, so will always be None for previous render
    // vnodes.
    props: Option<AnyBox>,
}

impl ComponentSpec {
    pub fn new<C: Component>(props: C::Properties) -> Self {
        // TODO: move this logic to component defintion / DynamicComponent trait.
        let init = |props: AnyBox, ctx: Context<AnyBox>| -> (BoxedComponent, VNode<AnyBox>) {
            let real_props = *(props
                .downcast::<C::Properties>()
                .expect("Invalid property type"));
            let real_context = ctx.into_nested();
            let c = C::init(real_props, real_context);
            let node = c.render();

            (Box::new(c), node.map(into_any_box))
        };

        let props = into_any_box(props);
        Self {
            constructor: ComponentConstructor(init),
            props: Some(props),
        }
    }

    #[inline]
    pub fn is_same_constructor(&self, other: &Self) -> bool {
        // TODO: figure out if this actually works as intended.
        // Otherwise we will need to add a TypeId to VComponent
        self.constructor.0 as usize == other.constructor.0 as usize
    }
}

impl std::fmt::Debug for ComponentSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentSpec")
            .field("props", &self.props)
            .finish()
    }
}

#[must_use]
struct ComponentBorrowFinisher {
    id: ComponentId,
}

impl ComponentBorrowFinisher {
    fn return_component(self, manager: &mut ComponentManager, comp: DynamicComponentState) {
        manager.components[ComponentManager::id_to_index(self.id)] = Some(comp);
    }
}

struct ComponentManager {
    // TODO: prevent need for option and unwraps. (?)
    components: Vec<Option<DynamicComponentState>>,
    idle: Vec<ComponentId>,
}

impl ComponentManager {
    fn new() -> Self {
        Self {
            components: Vec::new(),
            idle: Vec::new(),
        }
    }

    #[inline]
    fn id_to_index(id: ComponentId) -> usize {
        id.0 as usize - 1
    }

    #[inline]
    fn index_to_id(index: usize) -> ComponentId {
        ComponentId(index as u32 + 1)
    }

    fn register_component(&mut self, state: DynamicComponentState) -> ComponentId {
        if let Some(old_id) = self.idle.pop() {
            self.components[Self::id_to_index(old_id)] = Some(state);
            old_id
        } else {
            let id = Self::index_to_id(self.components.len());
            self.components.push(Some(state));
            id
        }
    }

    fn reserve_id(&mut self) -> ComponentBorrowFinisher {
        let id = if let Some(old_id) = self.idle.pop() {
            old_id
        } else {
            let id = Self::index_to_id(self.components.len());
            self.components.push(None);
            id
        };
        ComponentBorrowFinisher { id }
    }

    fn get_mut(&mut self, id: ComponentId) -> Option<&mut DynamicComponentState> {
        let real_id = Self::id_to_index(id);
        self.components.get_mut(real_id).and_then(|x| x.as_mut())
    }

    fn remove(&mut self, id: ComponentId) -> Option<DynamicComponentState> {
        let index = Self::id_to_index(id);
        self.idle.push(id);
        self.components.get_mut(index).and_then(|x| x.take())
    }

    fn borrow(
        &mut self,
        id: ComponentId,
    ) -> Option<(DynamicComponentState, ComponentBorrowFinisher)> {
        self.components[Self::id_to_index(id)]
            .take()
            .map(|x| (x, ComponentBorrowFinisher { id }))
    }
}

// struct DynamicComponent {
//     id: ComponentId,
//     type_id: TypeId,
//     component: Box<dyn Any>,
// }

pub(crate) struct AppState<C: Component> {
    window: web_sys::Window,
    document: web_sys::Document,
    component: C,
    parent: web_sys::Element,
    vnode: VNode<C::Msg>,
    // Render callback scheduled with requestAnimationFrame.
    animation_callback: wasm_bindgen::closure::Closure<dyn FnMut()>,
    /// Flag to enable batching and prevent redundant renders.
    /// Set to true when a render has been scheduled with requestAnimationFrame,
    /// and set to false after a render.
    root_render_queued: bool,

    routing_mapper: Option<Box<dyn Fn(String) -> Option<C::Msg>>>,

    pub event_manager: EventManager<C>,
    component_manager: ComponentManager,

    child_render_queue: Vec<ComponentId>,
}

type SkipRender = bool;

pub(crate) trait RenderContext<M> {
    fn create_element(&self, tag: dom::Tag) -> web_sys::Element;
    fn create_text_node(&self, text: &str) -> web_sys::Node;
    fn build_listener(&mut self, handler: EventHandler<M>) -> (EventCallbackId, &js_sys::Function);
    fn remove_listener(&mut self, id: EventCallbackId);
    fn get_listener_closure(&mut self, id: EventCallbackId) -> Option<&js_sys::Function>;
    fn mount_component<'a, 'b>(
        &'a mut self,
        comp: &'b mut VComponent,
        parent: &web_sys::Element,
        next_sibling: Option<&web_sys::Node>,
    ) -> Option<web_sys::Node>;
    fn remove_component(&mut self, id: ComponentId);
}

struct MainRenderContext<'a, C: Component> {
    app: &'a mut AppState<C>,
}

impl<'a, C: Component> RenderContext<C::Msg> for MainRenderContext<'a, C> {
    fn create_element(&self, tag: dom::Tag) -> web_sys::Element {
        self.app
            .document
            .create_element(tag.as_str())
            .expect("Could not create tag")
    }

    fn create_text_node(&self, text: &str) -> web_sys::Node {
        self.app.document.create_text_node(text).unchecked_into()
    }

    fn build_listener(
        &mut self,
        handler: EventHandler<C::Msg>,
    ) -> (EventCallbackId, &js_sys::Function) {
        self.app.event_manager.build(ManagedEvent::Root(handler))
    }

    fn remove_listener(&mut self, id: EventCallbackId) {
        self.app.event_manager.recycle(id);
    }

    fn get_listener_closure(&mut self, id: EventCallbackId) -> Option<&js_sys::Function> {
        self.app.event_manager.get_closure_fn(id)
    }

    fn mount_component<'a1, 'b>(
        &'a1 mut self,
        comp: &'b mut VComponent,
        parent: &web_sys::Element,
        next_sibling: Option<&web_sys::Node>,
    ) -> Option<web_sys::Node> {
        self.app.mount_component(comp, parent, next_sibling)
    }

    fn remove_component(&mut self, id: ComponentId) {
        self.app.remove_component(id);
    }
}

struct NestedRenderContext<'a, C: Component> {
    // TODO: use app directly
    component_id: ComponentId,
    main: MainRenderContext<'a, C>,
}

impl<'a, C: Component> RenderContext<AnyBox> for NestedRenderContext<'a, C> {
    #[inline]
    fn create_element(&self, tag: dom::Tag) -> web_sys::Element {
        self.main.create_element(tag)
    }

    #[inline]
    fn create_text_node(&self, text: &str) -> Node {
        self.main.create_text_node(text)
    }

    #[inline]
    fn build_listener(
        &mut self,
        handler: EventHandler<AnyBox>,
    ) -> (EventCallbackId, &js_sys::Function) {
        self.main.app.event_manager.build(ManagedEvent::Child {
            id: self.component_id,
            handler,
        })
    }

    #[inline]
    fn remove_listener(&mut self, id: EventCallbackId) {
        self.main.app.event_manager.recycle(id);
    }

    fn get_listener_closure(&mut self, id: EventCallbackId) -> Option<&js_sys::Function> {
        self.main.app.event_manager.get_closure_fn(id)
    }

    fn mount_component<'a1, 'b>(
        &'a1 mut self,
        comp: &'b mut VComponent,
        parent: &web_sys::Element,
        next_sibling: Option<&web_sys::Node>,
    ) -> Option<web_sys::Node> {
        self.main.app.mount_component(comp, parent, next_sibling)
    }

    fn remove_component(&mut self, id: ComponentId) {
        self.main.app.remove_component(id);
    }
}

impl<C: Component> AppState<C> {
    // TODO: prevent code duplication with Self::mount_component ?
    fn render_child(&mut self, component_id: ComponentId) -> Option<()> {
        let (mut state, finisher) = self.component_manager.borrow(component_id)?;

        let mut new_vnode = state.component.render();
        let mut render_context = NestedRenderContext {
            component_id,
            main: MainRenderContext { app: self },
        };

        let mut old_vnode = VNode::Empty;
        std::mem::swap(&mut state.last_vnode, &mut old_vnode);

        let new_node = patch(
            &mut render_context,
            &state.parent,
            state.next_sibling.as_ref(),
            old_vnode,
            &mut new_vnode,
        );
        state.last_vnode = new_vnode;
        state.node = new_node;
        finisher.return_component(&mut self.component_manager, state);

        None
    }

    fn mount_component<'a, 'b, 'c>(
        &'a mut self,
        vcomp: &'b mut VComponent,
        parent: &web_sys::Element,
        next_sibling: Option<&web_sys::Node>,
    ) -> Option<web_sys::Node> {
        if vcomp.id.0 > 0 {
            // Old component.
            let (mut state, finisher) = self
                .component_manager
                .borrow(vcomp.id)
                .expect("Component has disappeared");

            // NOTE: Effect::SkipCheck checking is done in the DynamicComponent
            // implementation.
            // This id done to prevent a second dynamic dispatch.
            // Instead, DynamicComponent::on_prop_change returns an Option<VNode>
            // with the rendered new VNode, if required.

            // FIXME: apply effect.

            // let new_props = vcomp.spec.props.take().unwrap_or_else(|| Box::new(()));
            // let mut effect = Effect::None;
            // let context = Context::new_direct(&mut effect).into_nested();
            // FIXME: figure out how to handle updated props.
            // let vnode_opt = state.component.on_property_change(new_props, context);
            let vnode_opt = Some(state.component.render());

            let node = if let Some(mut new_vnode) = vnode_opt {
                let mut render_context = NestedRenderContext {
                    component_id: vcomp.id,
                    main: MainRenderContext { app: self },
                };

                let mut old_vnode = VNode::Empty;
                std::mem::swap(&mut state.last_vnode, &mut old_vnode);

                let new_node = patch(
                    &mut render_context,
                    &state.parent,
                    state.next_sibling.as_ref(),
                    old_vnode,
                    &mut new_vnode,
                );
                state.last_vnode = new_vnode;
                state.node = new_node.clone();
                new_node
            } else {
                state.node.clone()
            };

            finisher.return_component(&mut self.component_manager, state);
            node
        } else {
            let props = vcomp.spec.props.take().unwrap_or_else(|| Box::new(()));

            let mut effect = Effect::None;
            let context = Context::new_direct(&mut effect).into_nested();

            let (component, mut vnode) = vcomp.spec.constructor.call(props, context);

            // TODO: rework the control flow so we don't have to reserve the
            // component id first.

            let finisher = self.component_manager.reserve_id();

            let mut render_context = NestedRenderContext {
                component_id: finisher.id,
                main: MainRenderContext { app: self },
            };

            let node = patch(
                &mut render_context,
                &parent,
                next_sibling,
                VNode::Empty,
                &mut vnode,
            );

            let state = ComponentState2 {
                component,
                last_vnode: vnode,
                parent: parent.clone(),
                next_sibling: next_sibling.cloned(),
                node: node.clone(),
            };
            vcomp.id = finisher.id;
            // self.apply_child_effect(finisher.id, effect, handle);
            // FIXME: apply effect.
            finisher.return_component(&mut self.component_manager, state);
            node
        }
    }

    fn remove_component(&mut self, id: ComponentId) {
        if let Some(state) = self.component_manager.remove(id) {
            if let Some(node) = state.node {
                state.parent.remove_child(&node).ok();
            }
        }
    }

    fn apply_effect(&mut self, effect: Effect<C::Msg>, handle: &AppHandle<C>) -> SkipRender {
        match effect {
            Effect::None => false,
            Effect::SkipRender => true,
            Effect::Navigate(path) => {
                self.window
                    .history()
                    .expect("Could not get window.history")
                    .push_state_with_url(&JsValue::undefined(), "", Some(&path))
                    .expect("Could not push history state");

                if let Some(mapper) = &self.routing_mapper {
                    if let Some(msg) = mapper(path) {
                        self.update(msg, handle)
                    }
                }

                false
            }
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
                            handle2.update(Task::Root(msg));
                        }
                        (Some(msg), None) => {
                            handle2.update(Task::Root(msg));
                        }
                        _ => {
                            // Either no message, or cancelled.
                        }
                    }
                });
                false
            }
            Effect::Subscription { stream, guard } => {
                let handle2 = handle.clone();
                // TODO: cancellation guard.
                wasm_bindgen_futures::spawn_local(async move {
                    let mut stream = stream;

                    while let Some(msg) = stream.next().await {
                        let is_cancelled =
                            guard.as_ref().map(|g| g.is_cancelled()).unwrap_or(false);
                        if is_cancelled {
                            break;
                        }
                        handle2.update(Task::Root(msg));
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
            Effect::Nested { effect: _effect } => {
                unreachable!()
            }
        }
    }

    // TODO: prevent duplication with Self::apply_effect?
    fn apply_child_effect(
        &mut self,
        component_id: ComponentId,
        effect: Effect<AnyBox>,
        handle: &AppHandle<C>,
    ) -> SkipRender {
        match effect {
            Effect::None => false,
            Effect::SkipRender => true,
            Effect::Navigate(path) => {
                self.window
                    .history()
                    .expect("Could not get window.history")
                    .push_state_with_url(&JsValue::undefined(), "", Some(&path))
                    .expect("Could not push history state");

                if let Some(mapper) = &self.routing_mapper {
                    if let Some(msg) = mapper(path) {
                        self.update(msg, handle)
                    }
                }

                false
            }
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
                            handle2.update(Task::Child { component_id, msg });
                        }
                        (Some(msg), None) => {
                            handle2.update(Task::Child { component_id, msg });
                        }
                        _ => {
                            // Either no message, or cancelled.
                        }
                    }
                });
                false
            }
            Effect::Subscription { stream, guard } => {
                let handle2 = handle.clone();
                // TODO: cancellation guard.
                wasm_bindgen_futures::spawn_local(async move {
                    let mut stream = stream;

                    while let Some(msg) = stream.next().await {
                        let is_cancelled =
                            guard.as_ref().map(|g| g.is_cancelled()).unwrap_or(false);
                        if is_cancelled {
                            break;
                        }
                        handle2.update(Task::Child { component_id, msg });
                    }
                });
                false
            }
            Effect::Multi(items) => {
                let mut skip = false;
                for item in items {
                    if self.apply_child_effect(component_id, item, handle) {
                        skip = true;
                    }
                }
                skip
            }
            Effect::Nested { effect } => self.apply_child_effect(component_id, *effect, handle),
        }
    }

    /// Schedule a re-render via requestAnimationFrame.
    /// If no child component id is given, the update is for the root.
    fn schedule_render_if_needed(&mut self, component: Option<ComponentId>) {
        let needs_schedule = self.child_render_queue.is_empty() && !self.root_render_queued;

        if let Some(id) = component {
            self.child_render_queue.push(id);
        } else {
            self.root_render_queued = true;
        }

        if needs_schedule {
            self.window
                .request_animation_frame(self.animation_callback.as_ref().unchecked_ref())
                .ok();
        }
    }

    fn update(&mut self, msg: C::Msg, handle: &AppHandle<C>) {
        // let start = crate::now();

        let mut effect = Effect::None;

        let context = Context::new_direct(&mut effect);
        self.component.update(msg, context);

        let skip_render = self.apply_effect(effect, handle);
        if !skip_render {
            self.schedule_render_if_needed(None);
        }

        // let time = crate::now() - start;
        // trace!(?time, "updated");
    }

    fn update_child(&mut self, component_id: ComponentId, msg: AnyBox, handle: &AppHandle<C>) {
        // let start = crate::now();

        let state = self
            .component_manager
            .get_mut(component_id)
            .expect("Component disappeared");

        let mut effect = Effect::None;
        let context = Context::new_direct(&mut effect).into_nested();
        state.component.update(msg, context);

        let skip_render = self.apply_child_effect(component_id, effect, handle);
        if !skip_render {
            self.schedule_render_if_needed(Some(component_id));
        }

        // let time = crate::now() - start;
        // trace!(?time, "updated");
    }

    fn render(&mut self) {
        // let start = crate::now();

        // If only a single child render is queued, we only re-render the child.
        // Otherwise the whole tree.
        // FIXME: be smarter here by determining the highest ancestors that need
        // re-render.
        // trace!(?self.root_render_queued, children=?self.child_render_queue.len(), "render start");
        if !self.root_render_queued && self.child_render_queue.len() == 1 {
            if let Some(id) = self.child_render_queue.pop() {
                self.render_child(id);
                // let time = crate::now() - start;
                // trace!(?time, "rendered");
                return;
            }
        }

        let mut new_vnode = self.component.render();
        let mut old_vnode = VNode::Empty;
        std::mem::swap(&mut old_vnode, &mut self.vnode);

        let parent = self.parent.clone();

        let mut context = MainRenderContext { app: self };
        super::patch::patch(&mut context, &parent, None, old_vnode, &mut new_vnode);
        self.vnode = new_vnode;
        self.root_render_queued = false;
        self.child_render_queue.clear();

        // let time = crate::now() - start;
        // trace!(?time, "rendered");
    }
}

enum Task<M> {
    Root(M),
    Child {
        component_id: ComponentId,
        msg: AnyBox,
    },
}

pub(crate) struct AppHandle<C: Component> {
    state: Rc<RefCell<AppState<C>>>,
    task_queue: Rc<RefCell<VecDeque<Task<C::Msg>>>>,
}

impl<C: Component> Clone for AppHandle<C> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            task_queue: self.task_queue.clone(),
        }
    }
}

impl<C: Component> AppHandle<C> {
    fn render(&self) {
        self.state.borrow_mut().render();
    }

    pub fn handle_event(&self, id: EventCallbackId, event: web_sys::Event) -> Option<()> {
        let mut s = self.state.borrow_mut();
        let handler = s.event_manager.get_handler(id)?;
        match handler {
            super::event_manager::ManagedEvent::Root(handler) => {
                let msg = handler.invoke(event)?;
                s.update(msg, self);
            }
            super::event_manager::ManagedEvent::Child { id, handler } => {
                let msg = handler.invoke(event)?;
                s.update_child(id, msg, self);
            }
        }

        None
    }

    fn update(&self, task: Task<C::Msg>) {
        self.task_queue.borrow_mut().push_back(task);

        if let Ok(mut borrow) = self.state.try_borrow_mut() {
            // TODO: get all at once!
            while let Some(task) = self.task_queue.borrow_mut().pop_front() {
                match task {
                    Task::Root(msg) => {
                        borrow.update(msg, self);
                    }
                    Task::Child { component_id, msg } => {
                        borrow.update_child(component_id, msg, self);
                    }
                }
            }
        }
    }

    pub fn boot(
        props: C::Properties,
        parent: web_sys::Element,
        route_mapper: Option<Box<dyn Fn(String) -> Option<C::Msg>>>,
    ) {
        let window = web_sys::window().expect("Could not retrieve window");
        let document = window.document().expect("Could not get document");

        let mut effect = Effect::None;
        let context = Context::new_direct(&mut effect);
        let component = C::init(props, context);

        let state = Rc::new(RefCell::new(AppState {
            window,
            document,
            component,
            parent,
            vnode: VNode::Empty,
            root_render_queued: false,
            child_render_queue: Vec::new(),
            // We first initialize the state with fake callbacks, since the real
            // ones need the Rc<> reference.
            animation_callback: wasm_bindgen::closure::Closure::wrap(Box::new(|| {})),
            routing_mapper: route_mapper,
            event_manager: EventManager::new(),
            component_manager: ComponentManager::new(),
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
