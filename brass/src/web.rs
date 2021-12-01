//! Helpers for interacting with the browser environment.

use js_sys::JsString;
use wasm_bindgen::JsCast;

/// Defines an enum that maps to plain string values and provides a cache
/// of `JsString`s for interaction with the dom.
///
/// This is useful for commonly used strings to prevent the overhead of
/// constant string re-encoding (UTF8 => UTF16 conversion) or hashing (in case
/// the wasm_bindgen interning feature is used).
#[macro_export]
macro_rules! make_str_enum {

    (
        $enum_name:ident {
            $( $name:ident = $value:literal, )*
        }
    ) => {
        #[repr(u16)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $enum_name {
            $( $name, )*
        }

        impl $enum_name {

            /// Convert to a string.
            pub fn as_str(self) -> &'static str {
                match self {
                    $(
                        Self::$name => $value,
                    )*
                }
            }

            // #[cfg(target = "wasm32-unknown-unknown")]
            pub fn as_js_string(self) -> &'static js_sys::JsString {

                match self {
                $(
                    Self::$name => {
                        static mut VALUE: once_cell::unsync::OnceCell<js_sys::JsString> = once_cell::unsync::OnceCell::new();
                        unsafe {
                            VALUE.get_or_init(|| wasm_bindgen::JsCast::unchecked_into(wasm_bindgen::JsValue::from(self.as_str())))
                        }
                    }
                 )*

                }
            }
        }


        impl From<$enum_name> for $crate::DomStr<'static> {
            fn from(value: $enum_name) -> Self {
                Self::JsStr(value.as_js_string())
            }
        }
    };
}

use crate::dom::{Attr, Event, Tag};

pub fn window() -> &'static web_sys::Window {
    static mut WINDOW: once_cell::unsync::Lazy<web_sys::Window> =
        once_cell::unsync::Lazy::new(|| web_sys::window().unwrap());

    unsafe { &WINDOW }
}

// pub fn document() -> &'static web_sys::Document {
//     static mut DOCUMENT: once_cell::unsync::Lazy<web_sys::Document> =
//         once_cell::unsync::Lazy::new(|| web_sys::window().unwrap().document().unwrap());
//     unsafe { &*DOCUMENT }
// }

pub struct CachedString(pub &'static str);

#[cfg(all(target_arch = "wasm", target_vendor = "unknown"))]
impl From<&'static CachedString> for DomStr<'static> {
    fn from(c: &'static CachedString) -> Self {
        static mut VALUE: once_cell::unsync::OnceCell<JsString> =
            once_cell::unsync::OnceCell::new();
        // SAFETY:
        // static mut is only unsafe due to multi-threading.
        // WASM doesn't support multithreading (yet), and even once it does it
        // will require special primitives for shared memory.
        // Since the function is restricted to the wasm(32)-unknown-unknown
        // target, this code will not exist on multi-threaded targets.
        let js = unsafe { VALUE.get_or_init(|| JsValue::from(c.0).unchecked_into()) };
        DomStr::JsStr(js)
    }
}

#[cfg(not(all(target_arch = "wasm", target_vendor = "unknown")))]
impl From<&'static CachedString> for DomStr<'static> {
    fn from(c: &'static CachedString) -> Self {
        DomStr::Str(c.0)
    }
}

#[macro_export]
macro_rules! dom_strings {
    (
    $(
        $name:ident = $value:literal,
    )*
    ) => {

        $(
            #[allow(non_upper_case_globals)]
            pub const $name: $crate::web::CachedString = $crate::web::CachedString($value);

        )*
    };
}

/// The `web-sys` crate offers a lot of APIs, but notably there are no setters
/// than can take an existing JS string reference.
/// This prevents improving performance with manual interning of strings, which
/// is a lot of the work being done by DOM creation.
///
/// These manually created Javascript helper functions provide additional
/// functons for common operations and do take existing [`js_sys::JsString`]
/// values.
#[wasm_bindgen::prelude::wasm_bindgen(inline_js = "
export function __brass_elem_set_attr_js_value(elem, attr, value) {
    elem.setAttribute(attr, value);
}

export function __brass_elem_set_attr_str_value(elem, attr, value) {
    elem.setAttribute(attr, value);
}

export function __brass_elem_remove_attr(elem, attr) {
    elem.removeAttribute(attr);
}

export function __brass_create_element(tag) {
    return document.createElement(tag);
}

export function __brass_add_event_listener(elem, event, listener) {
    elem.addEventListener(event, listener);
}

export function __brass_create_empty_node() {
    return document.createComment('')
}

export function __brass_create_text_node_str(value) {
    return document.createTextNode(value)
}

export function __brass_create_text_node_js(value) {
    return document.createTextNode(value)
}

export function __brass_set_text_data(node, value) {
    node.data = value;
}

export function __brass_class_list_add_js(elem, value) {
    elem.classList.add(value);
}

export function __brass_class_list_remove_js(elem, value) {
    elem.classList.remove(value);
}

export function __brass_class_list_add_str(elem, value) {
    elem.classList.add(value);
}

export function __brass_class_list_remove_str(elem, value) {
    elem.classList.remove(value);
}

export function __brass_class_set_js(elem, value) {
    elem.className = value;
}

export function __brass_elem_set_style_js_value(elem, style, value) {
    elem.style.setProperty(style, value)
}

export function __brass_elem_set_style_str_value(elem, style, value) {
    elem.style.setProperty(style, value)
}

export function __brass_document_fullscreen_element() {
    document.fullscreenElement;
}

")]
extern "C" {
    fn __brass_elem_set_attr_js_value(
        elem: &web_sys::Element,
        attr: &js_sys::JsString,
        value: &js_sys::JsString,
    );

    fn __brass_elem_set_attr_str_value(
        elem: &web_sys::Element,
        attr: &js_sys::JsString,
        value: &str,
    );

    fn __brass_elem_set_style_js_value(
        elem: &web_sys::Element,
        style: &js_sys::JsString,
        value: &js_sys::JsString,
    );

    fn __brass_elem_set_style_str_value(
        elem: &web_sys::Element,
        style: &js_sys::JsString,
        value: &str,
    );

    fn __brass_elem_remove_attr(elem: &web_sys::Element, attr: &js_sys::JsString);

    fn __brass_create_element(tag: &js_sys::JsString) -> wasm_bindgen::JsValue;

    fn __brass_add_event_listener(
        elem: &web_sys::EventTarget,
        event: &js_sys::JsString,
        listener: &js_sys::Function,
    );

    fn __brass_create_empty_node() -> web_sys::Node;

    fn __brass_create_text_node_str(value: &str) -> web_sys::Text;
    fn __brass_create_text_node_js(value: &JsString) -> web_sys::Text;

    fn __brass_set_text_data(node: &web_sys::Text, value: &JsString);

    fn __brass_class_set_js(elem: &web_sys::Element, value: &JsString);

    // ClassList
    fn __brass_class_list_add_js(elem: &web_sys::Element, value: &JsString);
    fn __brass_class_list_remove_js(elem: &web_sys::Element, value: &JsString);
    fn __brass_class_list_add_str(elem: &web_sys::Element, value: &str);
    fn __brass_class_list_remove_str(elem: &web_sys::Element, value: &str);

    fn __brass_document_fullscreen_element() -> Option<web_sys::Element>;
}

pub fn create_empty_node() -> web_sys::Node {
    __brass_create_empty_node()
}

static mut EMPTY_STRING: once_cell::unsync::OnceCell<JsString> = once_cell::unsync::OnceCell::new();

#[inline]
pub fn empty_string() -> &'static JsString {
    // Safety: safe in single-threaded context.
    // TODO: add #[cfg] flag to disable otherwise.
    unsafe { EMPTY_STRING.get_or_init(|| JsString::from("".to_string())) }
}

#[derive(Debug)]
pub enum DomStr<'a> {
    Str(&'a str),
    String(String),
    JsStr(&'a JsString),
    JsString(JsString),
}

impl<'a> From<&'a str> for DomStr<'a> {
    fn from(value: &'a str) -> Self {
        Self::Str(value)
    }
}

impl<'a> From<&'a String> for DomStr<'a> {
    fn from(value: &'a String) -> Self {
        Self::Str(value.as_str())
    }
}

impl<'a> From<String> for DomStr<'a> {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl<'a> From<&'a JsString> for DomStr<'a> {
    fn from(value: &'a JsString) -> Self {
        Self::JsStr(value)
    }
}

pub fn document_fullscreen_element() -> Option<web_sys::Element> {
    __brass_document_fullscreen_element()
}

//#[cfg(target = "wasm32-unknown-unknown")]
pub fn set_attribute(elem: &web_sys::Element, attr: Attr, value: DomStr<'_>) {
    match value {
        DomStr::Str(value) => {
            // TODO: use existing JsValue if `Str` is `Str::Repr::Interned`.
            __brass_elem_set_attr_str_value(elem, attr.as_js_string(), value);
        }
        DomStr::String(value) => {
            // TODO: use existing JsValue if `Str` is `Str::Repr::Interned`.
            __brass_elem_set_attr_str_value(elem, attr.as_js_string(), &value);
        }
        DomStr::JsStr(value) => {
            // TODO: use existing JsValue if `Str` is `Str::Repr::Interned`.
            __brass_elem_set_attr_js_value(elem, attr.as_js_string(), value);
        }
        DomStr::JsString(value) => {
            __brass_elem_set_attr_js_value(elem, attr.as_js_string(), &value);
        }
    }
}

pub fn set_style(elem: &web_sys::Element, style: crate::dom::Style, value: DomStr<'_>) {
    match value {
        DomStr::Str(value) => {
            // TODO: use existing JsValue if `Str` is `Str::Repr::Interned`.
            __brass_elem_set_style_str_value(elem, style.as_js_string(), value);
        }
        DomStr::String(value) => {
            // TODO: use existing JsValue if `Str` is `Str::Repr::Interned`.
            __brass_elem_set_style_str_value(elem, style.as_js_string(), &value);
        }
        DomStr::JsStr(value) => {
            // TODO: use existing JsValue if `Str` is `Str::Repr::Interned`.
            __brass_elem_set_style_js_value(elem, style.as_js_string(), value);
        }
        DomStr::JsString(value) => {
            __brass_elem_set_style_js_value(elem, style.as_js_string(), &value);
        }
    }
}

pub fn elem_set_class_js(elem: &web_sys::Element, value: &JsString) {
    __brass_class_set_js(elem, value);
}

pub fn elem_set_class<'a, I>(elem: &web_sys::Element, value: I)
where
    I: Into<DomStr<'a>>,
{
    match value.into() {
        DomStr::Str(value) => {
            elem.set_class_name(value);
        }
        DomStr::String(value) => {
            elem.set_class_name(&value);
        }
        DomStr::JsStr(value) => {
            elem_set_class_js(elem, value);
        }
        DomStr::JsString(value) => {
            elem_set_class_js(elem, &value);
        }
    }
}

pub fn elem_add_class_js(elem: &web_sys::Element, value: &JsString) {
    __brass_class_list_add_js(elem, value);
}

pub fn elem_add_class(elem: &web_sys::Element, value: &DomStr<'_>) {
    match value {
        DomStr::Str(value) => {
            __brass_class_list_add_str(elem, value);
        }
        DomStr::String(value) => {
            __brass_class_list_add_str(elem, &value);
        }
        DomStr::JsStr(value) => {
            __brass_class_list_add_js(elem, value);
        }
        DomStr::JsString(value) => {
            __brass_class_list_add_js(elem, &value);
        }
    }
}

pub fn elem_remove_class_js(elem: &web_sys::Element, value: &JsString) {
    __brass_class_list_remove_js(elem, value);
}

pub fn elem_remove_class(elem: &web_sys::Element, value: &DomStr<'_>) {
    match value {
        DomStr::Str(value) => {
            __brass_class_list_remove_str(elem, value);
        }
        DomStr::String(value) => {
            __brass_class_list_remove_str(elem, &value);
        }
        DomStr::JsStr(value) => {
            __brass_class_list_remove_js(elem, value);
        }
        DomStr::JsString(value) => {
            __brass_class_list_remove_js(elem, &value);
        }
    }
}

pub fn remove_attr(elem: &web_sys::Element, attr: Attr) {
    __brass_elem_remove_attr(elem, attr.as_js_string());
}

pub fn create_element(tag: Tag) -> web_sys::Element {
    __brass_create_element(tag.as_js_string()).unchecked_into()
}

pub fn create_text(value: DomStr<'_>) -> web_sys::Text {
    match value {
        DomStr::String(value) => __brass_create_text_node_str(&value).unchecked_into(),
        DomStr::Str(value) => __brass_create_text_node_str(&value).unchecked_into(),
        DomStr::JsStr(value) => __brass_create_text_node_js(value).unchecked_into(),
        DomStr::JsString(value) => __brass_create_text_node_js(&value).unchecked_into(),
    }
}

pub fn set_text_data(text: &web_sys::Text, value: &DomStr<'_>) {
    match value {
        DomStr::String(v) => {
            text.set_data(&v);
        }
        DomStr::Str(v) => {
            text.set_data(v);
        }
        DomStr::JsStr(v) => __brass_set_text_data(text, v),
        DomStr::JsString(v) => {
            __brass_set_text_data(text, v);
        }
    }
}

#[allow(unused)]
pub fn add_event_lister(target: &web_sys::EventTarget, event: Event, listener: &js_sys::Function) {
    unsafe {
        __brass_add_event_listener(target, event.as_js_string(), listener);
    }
}
