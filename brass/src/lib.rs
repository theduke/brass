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

use dom::TagBuilder;
pub use futures_signals as signal;

pub fn launch_component<C: Component + 'static>(
    parent: web_sys::Element,
    properties: C::Properties,
) {
    let tag = build_component::<C>(properties);
    tag.node.attach(&parent);
    std::mem::forget(tag);
}

pub fn launch<F: FnOnce() -> TagBuilder>(parent: web_sys::Element, render: F) {
    let node = render().build();
    node.attach(&parent);
    std::mem::forget(node);
}
