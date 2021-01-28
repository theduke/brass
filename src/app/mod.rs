mod component;
mod component_manager;
mod context;
mod effect;
mod event_manager;
mod handle;
mod state;

pub(crate) use self::{
    component::{ComponentConstructor, InstantiatedComponent},
    component_manager::ComponentId,
    event_manager::{ComponentEventHandler, EventCallbackId},
    state::AppState,
};

pub use self::{
    component::{Component, ShouldRender},
    context::Context,
    effect::{Callback, EffectGuard},
};

pub fn boot<C: Component>(props: C::Properties, node: web_sys::Element) {
    handle::AppHandle::boot::<C>(props, node)
}
