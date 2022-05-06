// mod strings;

// #![warn(unused_crate_dependencies)]

// NOTE: needs to be on top because it defines macros used elsewhere.
#[macro_use]
pub mod web;

pub mod context;

pub mod component;
pub mod dom;

pub mod effect;

pub use futures_signals as signal;

pub use self::web::DomStr;

#[cfg(feature = "macros")]
pub use brass_macros::view;

use component::{build_component, Component};

use context::{AppContext, AppContextRef};
use dom::Render;

pub fn launch_component<C: Component + 'static>(
    parent: web_sys::Element,
    properties: C::Properties,
) {
    launch(parent, move || build_component::<C>(properties));
}

pub fn launch<V: Render, F: FnOnce() -> V>(parent: web_sys::Element, render: F) -> AppContextRef {
    let mut ctx = AppContext::new();
    let view = ctx.with(move || {
        let view = render().render();
        view.attach(&parent);
        view
    });
    ctx.process_futures();
    std::mem::forget(view);

    ctx.leak_ref()
}
