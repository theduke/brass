pub mod dom;
pub mod vdom;

pub use vdom::{component::Component, VNode};

use wasm_bindgen::{JsCast};

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


pub fn boot<C: Component>(props: C::Properties, node: web_sys::Node) {
    vdom::component::ComponentHandle::<C>::boot(props, node)
}
