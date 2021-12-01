// mod strings;

// NOTE: needs to be on top because it defines macros used elsewhere.
#[macro_use]
pub mod web;

pub mod component;
pub mod dom;

pub mod effect;

pub use self::web::DomStr;

#[cfg(feature = "macros")]
pub use brass_macros::view;

use component::{build_component, Component};

use dom::View;
pub use futures_signals as signal;

pub fn launch_component<C: Component + 'static>(
    parent: web_sys::Element,
    properties: C::Properties,
) {
    let view = build_component::<C>(properties);
    view.attach(&parent);
    std::mem::forget(view);
}

pub fn launch<F: FnOnce() -> View>(parent: web_sys::Element, render: F) {
    let view = render();
    view.attach(&parent);
    std::mem::forget(view);
}
