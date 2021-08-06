pub mod render;

mod builder;
pub use builder::*;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    any::AnyBox,
    app::{ComponentConstructor, ComponentId, EventCallbackId},
    dom, Component,
};

/// Wrapper around a [`web_sys::Node`].
/// A fake JsValue (JsValue::UNDEFINED) is used for the empty state.
/// This reduces branching compared to using an Option<_>.
#[derive(Debug, Clone)]
pub(crate) struct OptionalNode(web_sys::Node);

impl AsRef<web_sys::Node> for OptionalNode {
    #[inline]
    fn as_ref(&self) -> &web_sys::Node {
        &self.0
    }
}

impl OptionalNode {
    #[inline]
    pub fn new(node: web_sys::Node) -> Self {
        Self(node)
    }

    #[inline]
    pub fn new_none() -> Self {
        use wasm_bindgen::JsCast;
        Self(wasm_bindgen::JsValue::UNDEFINED.unchecked_into())
    }

    // #[inline]
    // pub fn is_none(&self) -> bool {
    //     let v: &wasm_bindgen::JsValue = self.0.as_ref();
    //     v == &JsValue::UNDEFINED
    // }

    // #[inline]
    // pub fn as_option(&self) -> Option<&web_sys::Node> {
    //     if self.is_none() {
    //         None
    //     } else {
    //         Some(&self.0)
    //     }
    // }
}

/// Wrapper around a [`web_sys::Element`].
/// A fake JsValue (JsValue::UNDEFINED) is used for the empty state.
/// This reduces branching compared to using an Option<_>.
#[derive(Debug, Clone)]
pub(crate) struct OptionalElement(web_sys::Element);

impl AsRef<web_sys::Element> for OptionalElement {
    #[inline]
    fn as_ref(&self) -> &web_sys::Element {
        &self.0
    }
}

impl OptionalElement {
    #[inline]
    pub fn new(elem: web_sys::Element) -> Self {
        Self(elem)
    }

    #[inline]
    pub fn new_none() -> Self {
        use wasm_bindgen::JsCast;
        Self(wasm_bindgen::JsValue::UNDEFINED.unchecked_into())
    }

    // #[inline]
    // pub fn is_none(&self) -> bool {
    //     let v: &wasm_bindgen::JsValue = self.0.as_ref();
    //     v == &JsValue::UNDEFINED
    // }
}

// TODO: use cow or https://github.com/maciejhirsz/beef ?
#[derive(Clone, Debug)]
pub struct VText {
    value: String,
    node: OptionalNode,
}

impl VText {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            node: OptionalNode::new_none(),
        }
    }
}

impl<'a> From<&'a str> for VText {
    fn from(v: &'a str) -> Self {
        VText::new(v)
    }
}

impl<'a> From<&'a String> for VText {
    fn from(v: &'a String) -> Self {
        VText::new(v)
    }
}

impl From<String> for VText {
    fn from(v: String) -> Self {
        VText::new(v)
    }
}

pub type StaticEventHandler = fn(web_sys::Event) -> Option<AnyBox>;
// TODO: use box instead?
pub type ClosureEventHandler = Rc<dyn Fn(web_sys::Event) -> Option<AnyBox>>;

#[derive(Clone)]
pub enum EventCallback {
    Fn(StaticEventHandler),
    Closure(ClosureEventHandler),
}

impl std::fmt::Debug for EventCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EventCallback")
    }
}

impl EventCallback {
    pub fn invoke(&self, event: web_sys::Event) -> Option<AnyBox> {
        match self {
            EventCallback::Fn(f) => f(event),
            EventCallback::Closure(f) => (f)(event),
        }
    }
}

impl PartialEq for EventCallback {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EventCallback::Fn(f1), EventCallback::Fn(f2)) => f1 == f2,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
struct EventHandler {
    event: dom::Event,
    callback: EventCallback,
    callback_id: EventCallbackId,
}

// impl EventHandler {
//     pub(crate) fn new(event: dom::Event, handler: EventCallback) -> Self {
//         Self {
//             event,
//             callback: handler,
//             callback_id: EventCallbackId::new_null(),
//         }
//     }
// }

impl PartialEq for EventHandler {
    fn eq(&self, other: &Self) -> bool {
        self.event == other.event && self.callback == other.callback
    }
}

#[derive(Debug)]
pub struct VTag {
    tag: dom::Tag,
    // TODO: use a faster hash map and a better key.
    attributes: HashMap<dom::Attr, String>,
    children: Vec<VNode>,

    // TODO: should this be a u32 or a Rc<String> ?
    // TODO: implement keying support for the rendering
    key: Option<String>,
    event_handlers: Vec<EventHandler>,

    element: OptionalElement,
}

impl VTag {
    pub fn new(tag: dom::Tag) -> Self {
        Self {
            tag,
            attributes: HashMap::new(),
            children: Vec::new(),
            key: None,
            event_handlers: Vec::new(),
            element: OptionalElement::new_none(),
        }
    }
}

/// A reference to a DOM element.
///
/// Can be used inside of components to get access to the actual dom nodes
/// in lifecycle hooks.
#[derive(Clone)]
pub struct Ref(Rc<RefCell<Option<web_sys::Element>>>);

impl Ref {
    pub fn new() -> Self {
        Ref(Rc::new(RefCell::new(None)))
    }

    pub fn get(&self) -> Option<web_sys::Element> {
        self.0.try_borrow().ok()?.clone()
    }

    pub(crate) fn set(&self, elem: web_sys::Element) {
        self.0.replace(Some(elem));
    }

    pub(crate) fn take(&self) -> Option<web_sys::Element> {
        self.0.take()
    }
}

/// VDom element that enables getting a [`Ref`] to the [`web_sys::Element`] of
/// a tag.
pub struct VRef {
    pub(crate) tag: VTag,
    data: Ref,
}

impl VRef {
    fn set(&self, elem: web_sys::Element) {
        self.data.0.replace(Some(elem));
    }

    fn clear_ref(&self) {
        self.data.take();
    }

    fn swap_ref(&mut self, vref: Ref) {
        self.data = vref;
    }
}

impl std::fmt::Debug for VRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VRef")
            .field("tag", &self.tag)
            .field("ref", &())
            .finish()
    }
}

pub(crate) struct ComponentSpec {
    pub type_id: std::any::TypeId,
    pub constructor: ComponentConstructor,
    // Properties for the component.
    // Will be used during rendering, so will always be None for previous render
    // vnodes.
    // TODO: use Option<>  for components without properties to avoid allocations.
    pub props: Option<AnyBox>,
}

impl ComponentSpec {
    pub fn new<C: Component>(props: C::Properties) -> Self {
        Self {
            type_id: std::any::TypeId::of::<C>(),
            constructor: ComponentConstructor::new::<C>(),
            props: Some(Box::new(props)),
        }
    }
}

impl std::fmt::Debug for ComponentSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentSpec")
            .field("props", &self.props)
            .finish()
    }
}

#[derive(Debug)]
pub struct VComponent {
    pub(crate) spec: ComponentSpec,
    pub(crate) id: ComponentId,
}

impl VComponent {
    pub fn new<C: Component>(props: C::Properties) -> Self {
        Self {
            spec: ComponentSpec::new::<C>(props),
            id: ComponentId::NONE,
        }
    }

    pub fn is_same_constructor(&self, other: &Self) -> bool {
        self.spec.type_id == other.spec.type_id
    }
}

#[derive(Debug)]
pub enum VNode {
    Empty,
    Text(VText),
    Tag(VTag),
    Ref(VRef),
    Component(VComponent),
}

impl VNode {
    pub fn to_html(&self) -> String {
        let mut s = String::new();
        self.append_html(&mut s, 0);
        s
    }

    fn append_html(&self, s: &mut String, indent: usize) {
        match self {
            VNode::Empty => {}
            VNode::Text(t) => s.push_str(&t.value),
            VNode::Tag(tag) => {
                s.extend(std::iter::repeat(' ').take(indent));

                s.push('<');
                s.push_str(tag.tag.as_str());
                for (attr, value) in &tag.attributes {
                    s.push(' ');
                    s.push_str(attr.as_str());
                    s.push('=');
                    s.push_str(&value);
                }

                if tag.children.is_empty() {
                    s.push_str("/>");
                } else {
                    s.push('>');
                    let mut has_newlines = false;
                    for child in &tag.children {
                        let need_newline = match child {
                            VNode::Empty => false,
                            VNode::Text(_) => false,
                            VNode::Tag(_) => true,
                            VNode::Ref(_) => true,
                            VNode::Component(_) => false,
                        };

                        let child_indent = if need_newline { indent + 2 } else { 0 };
                        if need_newline {
                            has_newlines = true;
                            s.push('\n');
                        }
                        child.append_html(s, child_indent);
                    }
                    if has_newlines {
                        s.extend(std::iter::repeat(' ').take(indent))
                    }
                    s.push('<');
                    s.push('/');
                    s.push_str(tag.tag.as_str());
                    s.push('>');

                    s.push('\n');
                }
            }
            VNode::Ref(_) => todo!(),
            VNode::Component(_) => {}
        }
    }
}

impl Default for VNode {
    fn default() -> Self {
        Self::Empty
    }
}
