use futures::future::{AbortHandle, Abortable};
use futures_signals::signal_vec::{SignalVec, SignalVecExt, VecDiff};
use wasm_bindgen::{JsCast, JsValue};

use crate::{context::AppContext, web::create_empty_node};

use super::{view::RetainedView, AbortGuard, Render, View};

/// A view that is backed by a [`Signal`].
///
/// A future managing the signal runs on the render executor.
/// This proxy only stores the current rendered value to enable event handling
/// and cleanup.
pub struct SignalVecView(Box<Inner>);

struct Inner {
    fallback: Option<RetainedView>,
    marker: web_sys::Node,
    children: Vec<RetainedView>,
    _abort: AbortGuard,
    parent: Option<web_sys::Node>,
}

impl SignalVecView {
    pub(crate) fn replace_with(self, parent: &web_sys::Node, new_view: View) -> RetainedView {
        let new = new_view.into_retained();
        new.insert_before(parent, &self.0.marker);
        parent.remove_child(&self.0.marker).unwrap();
        for child in self.0.children {
            child.remove_from_parent(parent);
        }
        new
    }

    pub(crate) fn remove_from_parent(&self, parent: &web_sys::Node) {
        parent.remove_child(&self.0.marker).unwrap();
        for child in &self.0.children {
            child.remove_from_parent(parent);
        }
    }

    pub fn replace(&self, parent: &web_sys::Node, old_node: &web_sys::Node) {
        for child in self.0.children.iter().rev() {
            child.insert_before(parent, old_node);
        }
        parent.replace_child(&self.0.marker, old_node).unwrap();
    }

    pub(crate) fn attach(&self, parent: &web_sys::Node) {
        debug_assert!(self.0.children.is_empty());
        parent.append_child(&self.0.marker).unwrap();
    }

    pub(crate) fn insert_before(&self, parent: &web_sys::Node, before: &web_sys::Node) {
        for child in self.0.children.iter().rev() {
            child.insert_before(parent, before);
        }
        parent.insert_before(&self.0.marker, Some(before)).unwrap();
    }

    pub(crate) fn prepend_before_self(&self, parent: &web_sys::Node, new: &RetainedView) {
        if let Some(child) = self.0.children.first() {
            child.prepend_before_self(parent, new);
        } else if let Some(fb) = &self.0.fallback {
            fb.prepend_before_self(parent, new);
        } else {
            new.insert_before(parent, &self.0.marker);
        }
    }

    // pub(crate) fn children(&self) -> &[RetainedView] {
    //     &self.0.children
    // }

    pub(crate) fn new<T, S, O, R>(signal: S, render: R, fallback: Option<View>) -> Self
    where
        S: SignalVec<Item = T> + 'static,
        R: Fn(&T) -> O + 'static,
        O: Render,
    {
        let (handle, reg) = AbortHandle::new_pair();
        let mut inner = Box::new(Inner {
            marker: create_empty_node(),
            children: Vec::new(),
            fallback: fallback.map(|f| f.into_retained()),
            _abort: AbortGuard::new(handle),
            parent: None,
        });

        let f = {
            // SAFETY:
            // Creates a 'static reference to the inner data.
            // The reference is only given to a future which is spawned on
            // a thread-local executor. The future is guarded by the [AbortGuard]
            // owned by the struct.
            // Since the futures executor is single-threaded and only runs on
            // the same thread the, future can't run after the struct has been
            // dropped (since it will be aborted) beforehand.
            let state = unsafe { &mut *(inner.as_mut() as *mut Inner) };

            let mut fallback_visible = false;

            signal.for_each(move |patch| {
                let parent = if let Some(p) = &state.parent {
                    p
                } else if let Some(p) = state.marker.parent_node() {
                    state.parent = Some(p);
                    state.parent.as_ref().unwrap()
                } else {
                    tracing::error!(
                        "ViewSignalVec received an update but is not attached to a parent!"
                    );
                    return std::future::ready(());
                };


                // TODO use custom binding for efficiency?

                match patch {
                    VecDiff::Replace { values } => {
                        state.children.drain(..).for_each(|child| {
                            child.remove_from_parent(parent);
                        });

                        if values.is_empty() {
                            if !fallback_visible {
                                if let Some(e) = state.fallback.as_ref() {
                                    e.insert_before(parent, &state.marker);
                                    fallback_visible = true;
                                }
                            }
                        } else {
                            if fallback_visible {
                                state.fallback.as_ref().unwrap().remove_from_parent(parent);
                                fallback_visible = false;
                            }

                            for value in values {
                                let rendered = render(&value).render().into_retained();
                                rendered.insert_before(parent, &state.marker);
                                state.children.push(rendered);
                            }
                        }
                    }
                    VecDiff::InsertAt { index, value } => {
                        if fallback_visible {
                            state.fallback.as_ref().unwrap().remove_from_parent(parent);
                            fallback_visible = false;
                        }

                        let new_child = render(&value).render().into_retained();

                        if let Some(current) = state.children.get(index) {
                            current.prepend_before_self(parent, &new_child);
                            state.children.insert(index, new_child);
                        } else {
                            tracing::warn!("VecDiff::InsertAt with invalid index {index} - exceeds current length of {}", state.children.len());
                            new_child.insert_before(parent, &state.marker);
                            state.children.push(new_child);
                        }
                    }
                    VecDiff::UpdateAt { index, value } => {
                        let new_child = render(&value).render();
                        if let Some(old) = state.children.get_mut(index) {

                            let mut tmp = RetainedView::Placeholder(JsValue::NULL.unchecked_into());
                            std::mem::swap(old, &mut tmp);

                            let retained = tmp.replace_with(parent, new_child);
                            *old = retained;
                        } else {
                            tracing::warn!("invalid VecDiff::UpdateAt: index {index} does not exist");
                        }
                    }
                    VecDiff::RemoveAt { index } => {
                        if index < state.children.len() {
                            let old = state.children.remove(index);
                            old.remove_from_parent(parent);
                        }

                        if state.children.is_empty() && !fallback_visible {
                            if let Some(e) = state.fallback.as_ref() {
                                e.insert_before(parent, &state.marker);
                                fallback_visible = true;
                            }
                        }
                    }
                    VecDiff::Move {
                        old_index: _,
                        new_index: _,
                    } => {
                        todo!();

                    }
                    VecDiff::Push { value } => {
                        let child = render(&value).render().into_retained();
                        child.insert_before(parent, &state.marker);
                        state.children.push(child);

                        if fallback_visible {
                            state.fallback.as_ref().unwrap().remove_from_parent(parent);
                            fallback_visible = false;
                        }
                    }
                    VecDiff::Pop {} => {
                        if let Some(old) = state.children.pop() {
                            old.remove_from_parent(parent);
                        }

                        if state.children.is_empty() && !fallback_visible {
                            if let Some(e) = state.fallback.as_ref() {
                                e.insert_before(parent, &state.marker);
                                fallback_visible = true;
                            }
                        }
                    }
                    VecDiff::Clear {} => {
                        state.children.drain(..).for_each(|child| {
                            child.remove_from_parent(parent);
                        });

                        if !fallback_visible {
                            if let Some(e) = state.fallback.as_ref() {
                                e.insert_before(parent, &state.marker);
                                fallback_visible = true;
                            }
                        }
                    }
                }

                std::future::ready(())
            })
        };

        let abortable_future = Abortable::new(f, reg);
        AppContext::spawn_unguarded(async move {
            abortable_future.await.ok();
        });

        Self(inner)
    }
}
