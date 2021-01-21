#![recursion_limit = "512"]

pub mod util;

mod app;
pub mod dom;
pub mod vdom;

pub use app::{boot, Component, Context, ShouldRender};

type AnyBox = Box<dyn std::any::Any>;

fn into_any_box(value: impl std::any::Any) -> AnyBox {
    Box::new(value)
}

pub use vdom::VNode;

use wasm_bindgen::JsCast;

pub fn now() -> f64 {
    js_sys::Date::now()
}

pub fn url_path() -> String {
    web_sys::window().unwrap().location().pathname().unwrap()
}

pub fn input_event_value(ev: web_sys::Event) -> Option<String> {
    let v = ev
        .current_target()?
        .dyn_ref::<web_sys::HtmlInputElement>()?
        .value();
    Some(v)
}

pub fn textarea_input_value(ev: web_sys::Event) -> Option<String> {
    let v = ev
        .current_target()?
        .dyn_ref::<web_sys::HtmlTextAreaElement>()?
        .value();
    Some(v)
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

#[macro_export]
macro_rules! enable_props {
    ($prop:ty => $comp:ty) => {
        impl<M> copper::vdom::DomExtend<M> for $prop {
            fn extend(self, parent: &mut TagBuilder<M>) {
                let x = <$comp as copper::vdom::Component>::build(self);
                parent.add_child(x);
            }
        }
    };
}

#[macro_export]
macro_rules! rsx {
    ( $tag:ident $($extra:tt)* ) => {
        rsx!(
            !
            (
                $crate::vdom::TagBuilder::new($crate::dom::Tag::$tag)
            )
            $($extra)*
        )
    };

    (! ( $($code:tt)* ) $attr:ident = $value:literal $( $extra:tt )* ) => {
        rsx!(
            !
            ( $( $code )* .attr($crate::dom::Attr::$attr, $value) )
            $( $extra )*
        )
    };

    (! ( $($code:tt)* ) ) => {
        $($code)*
    };
}
