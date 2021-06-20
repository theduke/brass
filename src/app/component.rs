use std::rc::Rc;

use crate::{
    any::{any_box, AnyBox},
    vdom::render::DomRenderContext,
    VNode,
};

use super::{
    component_manager::ComponentId, context::Context, effect::EffectGuard, state::AppState,
};

pub type ShouldRender = bool;

pub struct RenderContext<C> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Component> RenderContext<C> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    pub fn callback<E, F>(&self, handler: F) -> crate::vdom::EventCallback
    where
        E: wasm_bindgen::JsCast + AsRef<web_sys::Event>,
        F: Fn(E) -> C::Msg + 'static,
    {
        crate::vdom::EventCallback::Closure(Rc::new(move |ev: web_sys::Event| {
            // TODO This expect can basically never happen due to the trait bound on E.
            // We could use JsCast::unchecked_into instead.
            // Keep this now just to be safe.
            match wasm_bindgen::JsCast::dyn_into(ev) {
                Ok(ev) => {
                    let msg = handler(ev);
                    Some(any_box(msg))
                }
                Err(err) => {
                    tracing::error!(?err, "Event callback received invalid event type");
                    None
                }
            }
        }))
    }

    pub fn callback_opt<E, F>(&self, handler: F) -> crate::vdom::EventCallback
    where
        E: wasm_bindgen::JsCast + AsRef<web_sys::Event>,
        F: (Fn(E) -> Option<C::Msg>) + 'static,
    {
        crate::vdom::EventCallback::Closure(Rc::new(move |ev: web_sys::Event| {
            // TODO This expect can basically never happen due to the trait bound on E.
            // We could use JsCast::unchecked_into instead.
            // Keep this now just to be safe.
            match wasm_bindgen::JsCast::dyn_into(ev) {
                Ok(ev) => {
                    if let Some(msg) = handler(ev) {
                        Some(any_box(msg))
                    } else {
                        None
                    }
                }
                Err(err) => {
                    tracing::error!(?err, "Event callback received invalid event type");
                    None
                }
            }
        }))
    }
}

pub trait Component: Sized + 'static {
    type Properties;
    type Msg: 'static;

    fn init(props: Self::Properties, ctx: &mut Context<Self::Msg>) -> Self;
    fn update(&mut self, msg: Self::Msg, ctx: &mut Context<Self::Msg>);
    fn render(&self, ctx: RenderContext<Self>) -> VNode;

    fn on_property_change(
        &mut self,
        props: Self::Properties,
        ctx: &mut Context<Self::Msg>,
    ) -> ShouldRender;

    /// Called immediately after a component has been attached to the DOM.
    #[allow(unused_variables)]
    fn on_render(&mut self, first_render: bool) {}

    /// Called before a component is removed from the DOM.
    ///
    /// Use this hook to clean up any DOM related state like manually attached
    /// event listeners.
    fn on_unmount(&mut self) {}

    /// Construct a new VNode during rendering.
    fn build(props: Self::Properties) -> VNode {
        crate::vdom::component::<Self>(props)
    }
}

type BoxedComponent = Box<dyn DynamicComponent>;

pub(crate) trait DynamicComponent {
    fn name(&self) -> &'static str;

    fn init(
        app: &mut AppState,
        id: ComponentId,
        props: AnyBox,
        parent: web_sys::Element,
        next_sibling: Option<web_sys::Node>,
    ) -> InstantiatedComponent
    where
        Self: Sized;

    fn dyn_render(
        &mut self,
        app: &mut AppState,
        state: &mut ComponentState,
    ) -> Option<web_sys::Node>;

    fn update(
        &mut self,
        app: &mut AppState,
        state: &mut ComponentState,
        msg: AnyBox,
    ) -> ShouldRender;

    fn remount(
        &mut self,
        app: &mut AppState,
        state: &mut ComponentState,
        new_props: AnyBox,
    ) -> Option<web_sys::Node>;
}

impl<C: Component> DynamicComponent for C {
    fn name(&self) -> &'static str {
        std::any::type_name::<C>()
    }

    /// Construct, initialize and render a component from boxed properties.
    fn init(
        app: &mut AppState,
        id: ComponentId,
        props: AnyBox,
        parent: web_sys::Element,
        next_sibling: Option<web_sys::Node>,
    ) -> InstantiatedComponent
    where
        Self: Sized,
    {
        let real_props = *props
            .downcast::<C::Properties>()
            .expect("Invalid property type");

        let (mut component, effect_guards) = {
            let mut context = Context::new(app, id);
            let c = C::init(real_props, &mut context);
            (c, context.take_effects())
        };
        let ctx = RenderContext::new();
        let mut vnode = component.render(ctx);

        let mut render_ctx = DomRenderContext::<C>::new(app, id);
        let node = render_ctx.patch(&parent, next_sibling.as_ref(), VNode::Empty, &mut vnode);

        // Call Component::on_render hook.
        component.on_render(true);

        InstantiatedComponent {
            component: Box::new(component),
            state: ComponentState {
                id,
                last_vnode: vnode,
                parent,
                next_sibling,
                node,
                effect_guards,
            },
        }
    }

    fn dyn_render(
        &mut self,
        app: &mut AppState,
        state: &mut ComponentState,
    ) -> Option<web_sys::Node> {
        let ctx = RenderContext::new();
        let mut vnode = self.render(ctx);
        let last_vnode = state.take_last_vnode();

        // trace!(?state, "dyn_render component {}", self.name());

        let mut render_ctx = DomRenderContext::<C>::new(app, state.id());
        let node = render_ctx.patch(
            &state.parent,
            state.next_sibling.as_ref(),
            last_vnode,
            &mut vnode,
        );

        // TODO: remove mapping
        state.last_vnode = vnode;
        state.node = node.clone();

        node
    }

    fn update(
        &mut self,
        app: &mut AppState,
        state: &mut ComponentState,
        msg: AnyBox,
    ) -> ShouldRender {
        let real_msg = *msg.downcast::<C::Msg>().expect("Invalid message type");

        let mut context = Context::new(app, state.id());
        self.update(real_msg, &mut context);

        !context.is_skip_render()
    }

    /// Remount a component with potentially changed properties.
    fn remount(
        &mut self,
        app: &mut AppState,
        state: &mut ComponentState,
        new_props: AnyBox,
    ) -> Option<web_sys::Node> {
        let real_props = *new_props
            .downcast::<C::Properties>()
            .expect("Invalid property type");

        let mut context = Context::new(app, state.id());
        let should_render = self.on_property_change(real_props, &mut context);

        if should_render && !context.is_skip_render() {
            self.dyn_render(app, state)
        } else {
            state.node.clone()
        }
    }
}

/// A wrapper around a function pointer that constructs a boxed component.
/// Used in [`VComponent`] for describing the component and by App to create it.
pub(crate) struct ComponentConstructor(
    fn(
        &mut AppState,
        ComponentId,
        AnyBox,
        web_sys::Element,
        Option<web_sys::Node>,
    ) -> InstantiatedComponent,
);

impl ComponentConstructor {
    pub fn new<C: DynamicComponent>() -> Self {
        Self(C::init)
    }

    #[inline]
    pub fn call(
        &self,
        app: &mut AppState,
        id: ComponentId,
        params: AnyBox,
        parent: web_sys::Element,
        next_sibling: Option<web_sys::Node>,
    ) -> InstantiatedComponent {
        (self.0)(app, id, params, parent, next_sibling)
    }
}

#[derive(Debug)]
pub(crate) struct ComponentState {
    id: ComponentId,
    /// VNode from the previous render.
    last_vnode: VNode,
    /// Parent dom element.
    parent: web_sys::Element,
    /// Next sibling. Needed for dom patching.
    next_sibling: Option<web_sys::Node>,
    node: Option<web_sys::Node>,

    // Effect guard for effects running for this component.
    // (scheduled with Context::run_unguarded etc)
    // Need to be cleaned up regularly.
    // TODO: auto-remove these guards on drop ?
    effect_guards: Vec<EffectGuard>,
}

impl ComponentState {
    #[inline]
    pub fn take_last_vnode(&mut self) -> VNode {
        std::mem::take(&mut self.last_vnode)
    }

    #[inline]
    pub fn id(&self) -> ComponentId {
        self.id
    }
}

pub(crate) struct InstantiatedComponent {
    /// The boxed component.
    component: BoxedComponent,
    state: ComponentState,
}

impl Drop for InstantiatedComponent {
    fn drop(&mut self) {
        if !self.state.effect_guards.is_empty() {
            tracing::warn!(
                "Component destroyed while '{}' unguarded effects are still active",
                self.state.effect_guards.len()
            );
        }
    }
}

impl InstantiatedComponent {
    // #[inline]
    // pub fn register_effect(&mut self, guard: EffectGuard) {
    //     self.state.effect_guards.push(guard);
    // }

    #[inline]
    pub fn update(&mut self, app: &mut AppState, msg: AnyBox) -> ShouldRender {
        self.component.update(app, &mut self.state, msg)
    }

    #[inline]
    pub fn render(&mut self, app: &mut AppState) -> Option<web_sys::Node> {
        self.component.dyn_render(app, &mut self.state)
    }

    #[inline]
    pub fn node(&self) -> Option<&web_sys::Node> {
        self.state.node.as_ref()
    }

    #[inline]
    pub fn remount(&mut self, app: &mut AppState, new_props: AnyBox) -> Option<web_sys::Node> {
        self.component.remount(app, &mut self.state, new_props)
    }

    pub fn remove_from_dom(mut self) -> Self {
        if let Some(node) = self.state.node.take() {
            self.state.parent.remove_child(&node).ok();
        }
        self
    }

    #[inline]
    pub fn state_mut(&mut self) -> &mut ComponentState {
        &mut self.state
    }
}