use core::panic;

use wasm_bindgen::JsCast;

use super::{EventCallback, OptionalElement, OptionalNode, VComponent, VNode, VRef, VTag};
use crate::{
    app::{AppState, ComponentEventHandler, ComponentId, EventCallbackId},
    dom, Component,
};

#[inline]
fn set_attribute(tag: dom::Tag, attribute: dom::Attr, value: &str, elem: &web_sys::Element) {
    match attribute {
        dom::Attr::Value if tag == dom::Tag::Input => {
            let input: &web_sys::HtmlInputElement = elem.unchecked_ref();
            input.set_value(value);
        }
        dom::Attr::Value if tag == dom::Tag::TextArea => {
            let input: &web_sys::HtmlTextAreaElement = elem.unchecked_ref();
            input.set_value(value);
        }
        dom::Attr::Checked if tag == dom::Tag::Input => {
            let input: &web_sys::HtmlInputElement = elem.unchecked_ref();
            // TODO: don't set if not required.
            input.set_checked(!value.is_empty());
        }
        _other => {
            elem.set_attribute(attribute.as_str(), value).ok();
        }
    }
}

pub struct DomRenderContext<'a, C: Component> {
    app: &'a mut AppState,
    component_id: ComponentId,
    _marker: std::marker::PhantomData<&'a C>,
}

impl<'a, C: Component> DomRenderContext<'a, C> {
    pub(crate) fn new(app: &'a mut AppState, component_id: ComponentId) -> Self {
        Self {
            app,
            component_id,
            _marker: std::marker::PhantomData,
        }
    }

    fn create_element(&self, tag: dom::Tag) -> web_sys::Element {
        self.app
            .document
            .create_element(tag.as_str())
            .expect("Could not create tag")
    }

    fn create_text_node(&self, text: &str) -> web_sys::Node {
        self.app.document.create_text_node(text).unchecked_into()
    }

    fn build_listener(&mut self, handler: EventCallback) -> (EventCallbackId, &js_sys::Function) {
        let ev = ComponentEventHandler::new(self.component_id, handler);
        self.app.event_manager.build(ev)
    }

    fn remove_listener(&mut self, id: EventCallbackId) {
        self.app.event_manager.recycle(id);
    }

    fn get_listener_closure(&mut self, id: EventCallbackId) -> Option<&js_sys::Function> {
        self.app.event_manager.get_closure_fn(id)
    }

    fn mount_component<'a1, 'b>(
        &'a1 mut self,
        comp: &'b mut VComponent,
        parent: &web_sys::Element,
        next_sibling: Option<&web_sys::Node>,
    ) -> Option<web_sys::Node> {
        self.app.mount_virtual_component(comp, parent, next_sibling)
    }

    // Patching and rendering.

    fn render_tag(
        &mut self,
        parent: &web_sys::Element,
        next_sibling: Option<&web_sys::Node>,
        tag: &mut VTag,
    ) -> web_sys::Element {
        let elem = self.create_element(tag.tag);

        // Set attributes.
        for (attribute, value) in &tag.attributes {
            set_attribute(tag.tag, *attribute, value, &elem);
        }

        // Add children.
        for child in &mut tag.children {
            self.render(&elem, None, child);
        }

        // Add listeners.
        if !tag.event_handlers.is_empty() {
            for listener in &mut tag.event_handlers {
                let (callback_id, fun) = self.build_listener(listener.callback.clone());
                listener.callback_id = callback_id;

                elem.add_event_listener_with_callback(listener.event.as_str(), fun)
                    .ok();
            }
        }

        parent.insert_before(&elem, next_sibling).ok();

        // TODO: avoid clone
        tag.element = OptionalElement::new(elem.clone());
        elem
    }

    fn render_ref(
        &mut self,
        parent: &web_sys::Element,
        next_sibling: Option<&web_sys::Node>,
        vref: &mut VRef,
    ) -> web_sys::Element {
        let elem = self.render_tag(parent, next_sibling, &mut vref.tag);
        vref.data.set(elem.clone());
        elem
    }

    // #[inline]
    // fn render_component<C: Component>(
    //     ctx: &mut AppState<C>,
    //     parent: &web_sys::Element,
    //     next_sibling: Option<&web_sys::Node>,
    //     comp: VComponent,
    // ) -> Option<web_sys::Node> {

    #[inline]
    fn render(
        &mut self,
        parent: &web_sys::Element,
        next_sibling: Option<&web_sys::Node>,
        vnode: &mut VNode,
    ) -> Option<web_sys::Node> {
        match vnode {
            VNode::Empty => None,
            VNode::Text(text) => {
                let node = self.create_text_node(&text.value);
                parent.insert_before(&node, next_sibling).ok();
                text.node = OptionalNode::new(node.clone());
                // TODO: prevent clones ?
                Some(node)
            }
            VNode::Tag(tag) => Some(self.render_tag(parent, next_sibling, tag).unchecked_into()),
            VNode::Ref(vref) => Some(self.render_ref(parent, next_sibling, vref).unchecked_into()),
            VNode::Component(comp) => {
                let node = self.mount_component(comp, parent, next_sibling.clone());
                if let Some(node) = node {
                    parent.insert_before(&node, next_sibling).ok();
                    Some(node)
                } else {
                    None
                }
            }
        }
    }

    fn patch_node_tag(&mut self, mut old: VTag, new: &mut VTag) {
        let elem = old.element.as_ref();

        // Ensure new attributes.
        for (key, value) in &new.attributes {
            if let Some(old_value) = old.attributes.get(key) {
                if old_value != value {
                    set_attribute(new.tag, *key, value, elem);
                }
            } else {
                set_attribute(new.tag, *key, value, elem);
            }
        }
        // Clear old attributes.
        for key in old.attributes.keys() {
            if !new.attributes.contains_key(key) {
                // Note: Ignore error for perf.
                elem.remove_attribute(key.as_str()).ok();
            }
        }

        // Handle listeners.
        for (index, new_listener) in new.event_handlers.iter_mut().enumerate() {
            if let Some(old_listener) = old.event_handlers.get_mut(index) {
                // See if we can reuse.
                if old_listener.event == new_listener.event
                    && old_listener.callback == new_listener.callback
                {
                    // Can reuse, just reuse the same id.
                    new_listener.callback_id = old_listener.callback_id;
                    old_listener.callback_id = EventCallbackId::new_null();
                    continue;
                } else {
                    // Free the old listener.
                    self.remove_listener(old_listener.callback_id);
                    if let Some(fun) = self.get_listener_closure(old_listener.callback_id) {
                        elem.remove_event_listener_with_callback(old_listener.event.as_str(), fun)
                            .ok();
                    }
                }
            }

            // Add new listener.
            let (callback_id, fun) = self.build_listener(new_listener.callback.clone());
            new_listener.callback_id = callback_id;
            elem.add_event_listener_with_callback(new_listener.event.as_str(), fun)
                .ok();
        }

        // Remove stale listeners.
        for listener in old
            .event_handlers
            .get(new.event_handlers.len()..)
            .unwrap_or_default()
        {
            if let Some(fun) = self.get_listener_closure(listener.callback_id) {
                elem.remove_event_listener_with_callback(listener.event.as_str(), fun)
                    .ok();
            }
            self.remove_listener(listener.callback_id);
        }

        // Handle children.
        let mut old_iter = old.children.into_iter();
        let mut new_iter = new.children.iter_mut();

        // let mut last_sibling = None;

        loop {
            match (old_iter.next(), new_iter.next()) {
                (None, None) => {
                    break;
                }
                (None, Some(child)) => {
                    self.render(elem, None, child);
                }
                (Some(old), None) => {
                    self.remove_node(old, elem);
                }
                (Some(old_tag), Some(new_tag)) => {
                    self.patch(elem, None, old_tag, new_tag);
                }
            }
        }

        new.element = old.element;
    }

    fn remove_component(
        &mut self,
        comp: VComponent,
        parent: &web_sys::Element,
        remove_from_dom: bool,
    ) {
        if let Some(mut c) = self.app.component_manager().remove(comp.id) {
            self.remove_node(c.state_mut().take_last_vnode(), parent);
            if remove_from_dom {
                c.remove_from_dom();
            }
        }
    }

    #[inline]
    fn remove_node(&mut self, node: VNode, parent: &web_sys::Element) {
        match node {
            VNode::Empty => {}
            VNode::Text(txt) => {
                parent.remove_child(txt.node.as_ref()).ok();
            }
            VNode::Tag(tag) => {
                self.remove_tag(tag, parent);
            }
            VNode::Ref(vref) => {
                vref.clear_ref();
                self.remove_tag(vref.tag, parent);
            }
            VNode::Component(c) => self.remove_component(c, parent, false),
        }
    }

    #[inline]
    fn remove_tag(&mut self, tag: VTag, parent: &web_sys::Element) {
        for listener in tag.event_handlers {
            self.remove_listener(listener.callback_id);
        }
        parent.remove_child(tag.element.as_ref()).ok();
    }

    fn patch_tag(
        &mut self,
        parent: &web_sys::Element,
        next_sibling: Option<&web_sys::Node>,
        old: VNode,
        new_tag: &mut VTag,
    ) -> web_sys::Node {
        if let VNode::Tag(old_tag) = old {
            if old_tag.tag == new_tag.tag {
                // Same tag, so we can patch
                self.patch_node_tag(old_tag, new_tag);
                let elem = new_tag.element.as_ref();
                let node: &web_sys::Node = elem.as_ref();
                node.clone()
            } else {
                self.remove_tag(old_tag, parent);
                // Must create a new tag.
                self.render_tag(parent, next_sibling, new_tag).into()
            }
        } else {
            self.remove_node(old, parent);
            // Must create a new tag.
            self.render_tag(parent, next_sibling, new_tag).into()
        }
    }

    pub fn patch(
        &mut self,
        parent: &web_sys::Element,
        next_sibling: Option<&web_sys::Node>,
        old: VNode,
        new: &mut VNode,
    ) -> Option<web_sys::Node> {
        let x = match new {
            VNode::Empty => {
                self.remove_node(old, parent);
                None
            }
            VNode::Text(new_txt) => {
                if let VNode::Text(old_txt) = old {
                    // TODO: prevent clones?
                    let node = old_txt.node.as_ref().clone();
                    if old_txt.value != new_txt.value {
                        node.set_node_value(Some(&new_txt.value));
                    }
                    new_txt.node = old_txt.node;
                    Some(node)
                } else {
                    self.remove_node(old, parent);

                    // Create new
                    let text_node = self.create_text_node(&new_txt.value);
                    parent.insert_before(&text_node, next_sibling).ok();
                    new_txt.node = OptionalNode::new(text_node.clone());
                    Some(text_node)
                }
            }
            VNode::Tag(new_tag) => Some(self.patch_tag(parent, next_sibling, old, new_tag)),
            VNode::Ref(new_ref) => {
                if let VNode::Ref(old_ref) = old {
                    let elem = self.patch_tag(
                        parent,
                        next_sibling,
                        VNode::Tag(old_ref.tag),
                        &mut new_ref.tag,
                    );
                    new_ref.swap_ref(old_ref.data);
                    new_ref.set(new_ref.tag.element.as_ref().clone());
                    Some(elem)
                } else {
                    let elem = self.patch_tag(parent, next_sibling, old, &mut new_ref.tag);
                    new_ref.data.set(new_ref.tag.element.as_ref().clone());
                    Some(elem)
                }
            }
            VNode::Component(new_comp) => {
                if let VNode::Component(old_comp) = old {
                    if old_comp.is_same_constructor(new_comp) {
                        new_comp.id = old_comp.id;
                        self.mount_component(new_comp, parent, next_sibling)
                    } else {
                        self.remove_component(old_comp, parent, true);
                        self.mount_component(new_comp, parent, next_sibling)
                    }
                } else {
                    self.remove_node(old, parent);
                    self.mount_component(new_comp, parent, next_sibling)
                }
            }
        };

        x
    }
}
