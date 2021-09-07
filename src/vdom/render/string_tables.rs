use std::cell::RefCell;

use wasm_bindgen::JsCast;

use crate::{
    dom::{Attr, Event, Tag},
    Str,
};

/// Initialize the static lookup tables.
pub fn setup() {
    unsafe {
        TABLES.initialize_data();
    }
}

/// The `web-sys` crate offers a lot of APIs, but notably there are no setters
/// than can take an existing JS string reference.
/// This prevents improving performance with manual interning of strings, which
/// is a lot of the work being done by DOM creation.
///
/// These manually created Javascript helper functions provide additional
/// setters for common operations and do take existing [`js_sys::JsString`]
/// values.
#[wasm_bindgen::prelude::wasm_bindgen(inline_js = "
export function __brass_elem_set_attr_js_value(elem, attr, value) {
    elem.setAttribute(attr, value);
}

export function __brass_elem_set_attr_str_value(elem, attr, value) {
    elem.setAttribute(attr, value);
}

export function __brass_create_element(tag) {
    return document.createElement(tag);
}

export function __brass_add_event_listener(elem, event, listener) {
    elem.addEventListener(event, listener);
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

    fn __brass_create_element(tag: &js_sys::JsString) -> wasm_bindgen::JsValue;

    fn __brass_add_event_listener(
        elem: &web_sys::EventTarget,
        event: &js_sys::JsString,
        listener: &js_sys::Function,
    );
}

pub fn set_element_attribute(elem: &web_sys::Element, attr: Attr, value: &Str) {
    unsafe {
        // TODO: use existing JsValue if `Str` is `Str::Repr::Interned`.
        __brass_elem_set_attr_str_value(elem, TABLES.attr(attr), value.as_str());
    }
}

pub fn create_element(tag: Tag) -> web_sys::Element {
    unsafe { __brass_create_element(TABLES.tag(tag)).unchecked_into() }
}

#[allow(unused)]
pub fn add_event_lister(target: &web_sys::EventTarget, event: Event, listener: &js_sys::Function) {
    unsafe {
        let event_name = TABLES.event(event);
        __brass_add_event_listener(target, event_name, listener);
    }
}

struct Tables {
    initialized: RefCell<bool>,

    attrs: [wasm_bindgen::JsValue; Attr::variant_count()],
    tags: [wasm_bindgen::JsValue; Tag::variant_count()],
    events: [wasm_bindgen::JsValue; Event::variant_count()],
}

impl Tables {
    const fn new_unitialized() -> Self {
        Self {
            initialized: RefCell::new(false),
            attrs: [wasm_bindgen::JsValue::NULL; Attr::variant_count()],
            tags: [wasm_bindgen::JsValue::NULL; Tag::variant_count()],
            events: [wasm_bindgen::JsValue::NULL; Event::variant_count()],
        }
    }

    fn initialize_data(&mut self) {
        let mut initialized = self.initialized.borrow_mut();
        if *initialized {
            return;
        }

        // Attr.
        debug_assert!(Attr::variant_count() < u16::MAX as usize);
        for num in 0..Attr::variant_count() {
            let attr = Attr::from_u16(num as u16).unwrap();
            let value = wasm_bindgen::JsValue::from_str(attr.as_str());
            self.attrs[num] = value;
        }

        // Tags.
        debug_assert!(Tag::variant_count() < u16::MAX as usize);
        for num in 0..Tag::variant_count() {
            let tag = Tag::from_u16(num as u16).unwrap();
            let value = wasm_bindgen::JsValue::from_str(tag.as_str());
            self.tags[num] = value;
        }

        // Event.
        debug_assert!(Event::variant_count() < u16::MAX as usize);
        for num in 0..Event::variant_count() {
            let event = Event::from_u16(num as u16).unwrap();
            let value = wasm_bindgen::JsValue::from_str(event.as_str());
            self.events[num] = value;
        }

        *initialized = true;
    }

    #[inline]
    fn attr(&self, attr: Attr) -> &js_sys::JsString {
        self.attrs[attr.as_u16() as usize].unchecked_ref()
    }

    #[inline]
    fn tag(&self, tag: Tag) -> &js_sys::JsString {
        self.tags[tag.as_u16() as usize].unchecked_ref()
    }

    #[inline]
    fn event(&self, event: Event) -> &js_sys::JsString {
        self.events[event.as_u16() as usize].unchecked_ref()
    }
}

static mut TABLES: Tables = Tables::new_unitialized();
