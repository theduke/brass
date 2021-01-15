pub mod dom;
pub mod vdom;

pub use vdom::{component::Component, VNode};

use wasm_bindgen::JsCast;

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

pub fn boot<C: Component>(props: C::Properties, node: web_sys::Element) {
    vdom::component::AppHandle::<C>::boot(props, node, None)
}

pub fn boot_routed<C, F>(props: C::Properties, node: web_sys::Element, mapper: F)
where
    C: Component,
    F: Fn(String) -> Option<C::Msg> + 'static,
{
    let mapper = Box::new(mapper);
    vdom::component::AppHandle::<C>::boot(props, node, Some(mapper))
}
