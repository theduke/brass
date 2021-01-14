pub mod dom;
pub mod vdom;

pub use vdom::{component::Component, Effect, VNode};

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Node};

pub fn now() -> f64 {
    js_sys::Date::now()
}

pub fn query_selector(sel: &str) -> Option<web_sys::Node> {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector(sel)
        .unwrap_or(None)
        .map(|x| x.unchecked_into())
}


pub fn nop<M>() -> Effect<M> {
    Effect::None
}


pub fn boot<C: Component>(props: C::Properties, node: web_sys::Node) {
    vdom::component::ComponentHandle::<C>::boot(props, node)
}
