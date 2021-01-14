use wasm_bindgen::JsCast;
use web_sys::Node;

use super::{
    component::{Component, ComponentState},
    event_manager::EventCallbackId,
    VNode, VTag, VText,
};
use crate::dom;

fn render_tag<C: Component>(
    ctx: &mut ComponentState<C>,
    parent: web_sys::Node,
    sibling: Option<web_sys::Node>,
    tag: &mut VTag<C::Msg>,
) -> web_sys::Node {
    let elem = ctx.create_element(tag.tag);
    let node: web_sys::Node = elem.clone().unchecked_into();

    // Set attributes.
    for (attribute, value) in &tag.attributes {
        set_attribute(tag.tag, elem.clone(), *attribute, value);
    }

    // Add children.
    for child in &mut tag.children {
        render(ctx, node.clone(), None, child);
    }

    // Add listeners.
    if !tag.listeners.is_empty() {
        for listener in &mut tag.listeners {
            let (callback_id, fun) = ctx.build_listener(listener.handler.clone());
            listener.callback_id = callback_id;

            let _ = elem.add_event_listener_with_callback(listener.event.as_str(), fun);
        }
    }

    // NOTE: ignoring error for perf.
    let _ = parent.insert_before(&elem, sibling.as_ref());

    tag.node = Some(node.clone());
    node
}

#[inline]
fn render<C: Component>(
    ctx: &mut ComponentState<C>,
    parent: web_sys::Node,
    sibling: Option<web_sys::Node>,
    vnode: &mut VNode<C::Msg>,
) -> Option<web_sys::Node> {
    match vnode {
        VNode::Empty => None,
        VNode::Text(text) => {
            let node = ctx.create_text_node(&text.value);
            // NOTE: ignoring error for perf.
            let _ = parent.insert_before(&node, sibling.as_ref());
            text.node = Some(node.clone());
            // TODO: prevent clones ?
            Some(node)
        }
        VNode::Tag(tag) => Some(render_tag(ctx, parent, sibling, tag)),
    }
}

#[inline]
fn set_attribute(tag: dom::Tag, elem: web_sys::Element, attribute: dom::Attr, value: &str) {
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
            tracing::trace!("setting to checked for input '{}'", value);
            let input: &web_sys::HtmlInputElement = elem.unchecked_ref();
            // TODO: don't set if not required.
            input.set_checked(!value.is_empty());
        }
        _other => {
            // Note: ignoring error for perf.
            elem.set_attribute(attribute.as_str(), value).ok();
        }
    }
}

fn patch_node_tag<C: Component>(
    ctx: &mut ComponentState<C>,
    mut old: VTag<C::Msg>,
    new: &mut VTag<C::Msg>,
) {
    let node = old.node.take().unwrap();

    let elem: &web_sys::Element = node.unchecked_ref();

    // Ensure new attributes.
    for (key, value) in &new.attributes {
        if let Some(old_value) = old.attributes.get(key) {
            if old_value != value {
                set_attribute(new.tag, elem.clone(), *key, value);
            }
        } else {
            set_attribute(new.tag, elem.clone(), *key, value);
        }
    }
    // Clear old attributes.
    for key in old.attributes.keys() {
        if !new.attributes.contains_key(key) {
            // Note: Ignore error for perf.
            let _ = elem.remove_attribute(key.as_str());
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
                ctx.event_manager.recycle(old_listener.callback_id);
                if let Some(fun) = ctx.event_manager.get_closure_fn(old_listener.callback_id) {
                    node.remove_event_listener_with_callback(old_listener.event.as_str(), fun)
                        .ok();
                }
            }
        }

        // Add new listener.
        let (callback_id, fun) = ctx.build_listener(new_listener.handler.clone());
        new_listener.callback_id = callback_id;
        let _ = node.add_event_listener_with_callback(new_listener.event.as_str(), fun);
    }

    // Remove stale listeners.
    for listener in &old.listeners[new.listeners.len()..] {
        ctx.event_manager.recycle(listener.callback_id);
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
                // TODO: prevent casting?
                last_sibling = render(ctx, node.clone(), None, child);
            }
            (Some(old), None) => {
                match old {
                    VNode::Empty => {}
                    VNode::Text(t) => {
                        if let Some(child_node) = &t.node {
                            // NOTE: ignore error for perf.
                            let _ = node.remove_child(child_node);
                        }
                    }
                    VNode::Tag(t) => {
                        if let Some(child_node) = &t.node {
                            // NOTE: ignore error for perf.
                            let _ = node.remove_child(&child_node);
                        }
                    }
                }
            }
            (Some(old_tag), Some(new_tag)) => {
                last_sibling = patch(ctx, node.clone(), last_sibling.clone(), old_tag, new_tag);
            }
        }
    }

    new.node = Some(node);
}

#[inline]
fn remove_node<C: Component>(ctx: &mut ComponentState<C>, node: VNode<C::Msg>, parent: Node) {
    match node {
        VNode::Empty => {}
        VNode::Text(txt) => {
            if let Some(node) = &txt.node {
                parent.remove_child(node).ok();
            }
        }
        VNode::Tag(tag) => {
            remove_tag(ctx, tag, parent);
        }
    }
}

#[inline]
fn remove_tag<C: Component>(ctx: &mut ComponentState<C>, tag: VTag<C::Msg>, parent: Node) {
    for listener in tag.listeners {
        ctx.event_manager.recycle(listener.callback_id);
    }
    if let Some(node) = &tag.node {
        parent.remove_child(node).ok();
    }
}

pub(super) fn patch<C: Component>(
    ctx: &mut ComponentState<C>,
    parent: Node,
    sibling: Option<Node>,
    old: VNode<C::Msg>,
    new: &mut VNode<C::Msg>,
) -> Option<web_sys::Node> {
    match (old, new) {
        (old, VNode::Empty) => {
            // TODO: recycle old node?
            if let VNode::Tag(tag) = old {
                remove_tag(ctx, tag, parent);
            }
            None
        }
        (old, VNode::Text(new_text)) => {
            if let VNode::Text(VText {
                node: Some(node),
                value: old_value,
            }) = old
            {
                if old_value != new_text.value {
                    node.set_node_value(Some(&new_text.value));
                }
                new_text.node = Some(node.clone());
                return Some(node);
            }

            // Remove old.
            remove_node(ctx, old, parent.clone());

            // Create new
            let text_node = ctx.create_text_node(&new_text.value);
            let next_sibling = sibling.as_ref().and_then(|s| s.next_sibling());
            // Note: ignore error for perf.
            let _ = parent.insert_before(&text_node, next_sibling.as_ref());
            new_text.node = Some(text_node.clone());
            Some(text_node)
        }
        (VNode::Empty, tag) => render(ctx, parent, None, tag),
        (VNode::Text(_), tag) => {
            // TODO: recycle text?
            render(ctx, parent, None, tag)
        }
        (VNode::Tag(old_tag), VNode::Tag(new_tag)) => {
            if let Some(_) = old_tag.node {
                if old_tag.tag == new_tag.tag {
                    // Same tag, so we can patch
                    patch_node_tag(ctx, old_tag, new_tag);
                    return new_tag.node.clone();
                } else {
                    remove_tag(ctx, old_tag, parent.clone());
                }
            }

            // Must create a new tag.
            Some(render_tag(ctx, parent, None, new_tag))
        }
    }
}
