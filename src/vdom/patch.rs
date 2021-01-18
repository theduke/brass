use wasm_bindgen::JsCast;

use super::{
    component::RenderContext, event_manager::EventCallbackId, OptionalElement, OptionalNode, VNode,
    VTag,
};
use crate::dom;

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

fn render_tag<M, R: RenderContext<M>>(
    ctx: &mut R,
    parent: &web_sys::Element,
    next_sibling: Option<&web_sys::Node>,
    tag: &mut VTag<M>,
) -> web_sys::Element {
    let elem = ctx.create_element(tag.tag);

    // Set attributes.
    for (attribute, value) in &tag.attributes {
        set_attribute(tag.tag, *attribute, value, &elem);
    }

    // Add children.
    for child in &mut tag.children {
        render(ctx, &elem, None, child);
    }

    // Add listeners.
    if !tag.listeners.is_empty() {
        for listener in &mut tag.listeners {
            let (callback_id, fun) = ctx.build_listener(listener.handler.clone());
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

// #[inline]
// fn render_component<C: Component>(
//     ctx: &mut AppState<C>,
//     parent: &web_sys::Element,
//     next_sibling: Option<&web_sys::Node>,
//     comp: VComponent,
// ) -> Option<web_sys::Node> {

#[inline]
fn render<M, R: RenderContext<M>>(
    ctx: &mut R,
    parent: &web_sys::Element,
    next_sibling: Option<&web_sys::Node>,
    vnode: &mut VNode<M>,
) -> Option<web_sys::Node> {
    match vnode {
        VNode::Empty => None,
        VNode::Text(text) => {
            let node = ctx.create_text_node(&text.value);
            parent.insert_before(&node, next_sibling).ok();
            text.node = OptionalNode::new(node.clone());
            // TODO: prevent clones ?
            Some(node)
        }
        VNode::Tag(tag) => Some(render_tag(ctx, parent, next_sibling, tag).unchecked_into()),
        VNode::Component(comp) => {
            let node = ctx.mount_component(comp, parent, next_sibling.clone());
            if let Some(node) = node {
                parent.insert_before(&node, next_sibling).ok();
                Some(node)
            } else {
                None
            }
        }
    }
}

fn patch_node_tag<M, R: RenderContext<M>>(ctx: &mut R, mut old: VTag<M>, new: &mut VTag<M>) {
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
    for (index, new_listener) in new.listeners.iter_mut().enumerate() {
        if let Some(old_listener) = old.listeners.get_mut(index) {
            // See if we can reuse.
            if old_listener.event == new_listener.event
                && old_listener.handler == new_listener.handler
            {
                // Can reuse, just reuse the same id.
                new_listener.callback_id = old_listener.callback_id;
                old_listener.callback_id = EventCallbackId::new_null();
                continue;
            } else {
                // Free the old listener.
                ctx.remove_listener(old_listener.callback_id);
                if let Some(fun) = ctx.get_listener_closure(old_listener.callback_id) {
                    elem.remove_event_listener_with_callback(old_listener.event.as_str(), fun)
                        .ok();
                }
            }
        }

        // Add new listener.
        let (callback_id, fun) = ctx.build_listener(new_listener.handler.clone());
        new_listener.callback_id = callback_id;
        elem.add_event_listener_with_callback(new_listener.event.as_str(), fun)
            .ok();
    }

    // Remove stale listeners.
    for listener in &old.listeners[new.listeners.len()..] {
        ctx.remove_listener(listener.callback_id);
    }

    // Handle children.
    let mut old_iter = old.children.into_iter();
    let mut new_iter = new.children.iter_mut();

    let mut last_sibling = None;

    loop {
        match (old_iter.next(), new_iter.next()) {
            (None, None) => {
                break;
            }
            (None, Some(child)) => {
                last_sibling = render(ctx, elem, None, child);
            }
            (Some(old), None) => {
                remove_node(ctx, old, elem);
            }
            (Some(old_tag), Some(new_tag)) => {
                last_sibling = patch(ctx, elem, None, old_tag, new_tag);
            }
        }
    }

    new.element = old.element;
}

#[inline]
fn remove_node<M, R: RenderContext<M>>(ctx: &mut R, node: VNode<M>, parent: &web_sys::Element) {
    match node {
        VNode::Empty => {}
        VNode::Text(txt) => {
            parent.remove_child(txt.node.as_ref()).ok();
        }
        VNode::Tag(tag) => {
            remove_tag(ctx, tag, parent);
        }

        VNode::Component(c) => {
            ctx.remove_component(c.id);
        }
    }
}

#[inline]
fn remove_tag<M, R: RenderContext<M>>(ctx: &mut R, tag: VTag<M>, parent: &web_sys::Element) {
    for listener in tag.listeners {
        ctx.remove_listener(listener.callback_id);
    }
    parent.remove_child(tag.element.as_ref()).ok();
}

pub(super) fn patch<'a, M, R: RenderContext<M>>(
    ctx: &mut R,
    parent: &web_sys::Element,
    next_sibling: Option<&web_sys::Node>,
    old: VNode<M>,
    new: &mut VNode<M>,
) -> Option<web_sys::Node> {
    match new {
        VNode::Empty => {
            remove_node(ctx, old, parent);
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
                remove_node(ctx, old, parent);

                // Create new
                let text_node = ctx.create_text_node(&new_txt.value);
                parent.insert_before(&text_node, next_sibling).ok();
                new_txt.node = OptionalNode::new(text_node.clone());
                Some(text_node)
            }
        }
        VNode::Tag(new_tag) => {
            if let VNode::Tag(old_tag) = old {
                if old_tag.tag == new_tag.tag {
                    // Same tag, so we can patch
                    patch_node_tag(ctx, old_tag, new_tag);
                    let elem = new_tag.element.as_ref();
                    let node: &web_sys::Node = elem.as_ref();
                    Some(node.clone())
                } else {
                    remove_tag(ctx, old_tag, parent);
                    // Must create a new tag.
                    Some(render_tag(ctx, parent, next_sibling, new_tag).into())
                }
            } else {
                remove_node(ctx, old, parent);
                // Must create a new tag.
                Some(render_tag(ctx, parent, next_sibling, new_tag).into())
            }
        }
        VNode::Component(new_comp) => {
            if let VNode::Component(old_comp) = old {
                if old_comp.is_same_constructor(new_comp) {
                    new_comp.id = old_comp.id;
                    ctx.mount_component(new_comp, parent, next_sibling)
                } else {
                    ctx.remove_component(old_comp.id);
                    ctx.mount_component(new_comp, parent, next_sibling)
                }
            } else {
                remove_node(ctx, old, parent);
                ctx.mount_component(new_comp, parent, next_sibling)
            }
        }
    }
}
