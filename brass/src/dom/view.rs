use wasm_bindgen::{JsCast, JsValue};

use crate::web::create_empty_node;

use super::{signal_vec_view::SignalVecView, signal_view::SignalView, Fragment, Node};

pub enum View {
    Empty,
    Node(Node),
    Fragment(Fragment),
    Signal(SignalView),
    SignalVec(SignalVecView),
}

impl Default for View {
    fn default() -> Self {
        Self::Empty
    }
}

impl From<()> for View {
    fn from(_: ()) -> Self {
        Self::Empty
    }
}

impl View {
    pub(crate) fn attach(&self, parent: &web_sys::Element) {
        match self {
            Self::Empty => {}
            Self::Signal(_inner) => {}
            Self::Node(n) => {
                n.attach(parent);
            }
            Self::Fragment(f) => {
                for item in &f.items {
                    item.attach(parent);
                }
            }
            Self::SignalVec(v) => {
                v.attach(parent);
            }
        }
    }

    pub fn as_node(&self) -> Option<&Node> {
        if let Self::Node(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn into_node(self) -> Option<Node> {
        if let Self::Node(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_fragment(&self) -> Option<&Fragment> {
        if let Self::Fragment(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the view is [`Empty`].
    ///
    /// [`Empty`]: View::Empty
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub(crate) fn into_retained(self) -> RetainedView {
        match self {
            View::Empty => RetainedView::Placeholder(create_empty_node()),
            View::Node(n) => RetainedView::Node(n),
            View::Fragment(frag) => RetainedView::Fragment(frag),
            View::Signal(sig) => RetainedView::Signal(sig),
            View::SignalVec(v) => RetainedView::SignalVec(v),
        }
    }
}

pub(crate) enum RetainedView {
    Placeholder(web_sys::Node),
    Node(Node),
    Fragment(Fragment),
    Signal(SignalView),
    SignalVec(SignalVecView),
}

impl RetainedView {
    pub fn new_placeholder() -> Self {
        Self::Placeholder(create_empty_node())
    }

    pub fn replace_with_mut(&mut self, parent: &web_sys::Node, new: View) {
        let mut tmp = Self::Placeholder(JsValue::NULL.unchecked_into());
        std::mem::swap(&mut tmp, self);
        *self = tmp.replace_with(parent, new);
    }

    pub fn replace_with(self, parent: &web_sys::Node, new: View) -> Self {
        match (self, new) {
            (p @ Self::Placeholder(_), View::Empty) => p,
            (Self::Placeholder(p), View::Node(n)) => {
                parent.replace_child(n.node(), &p).unwrap();
                Self::Node(n)
            }
            (Self::Placeholder(p), View::Signal(sig)) => {
                sig.replace(parent, &p);
                Self::Signal(sig)
            }
            (Self::Placeholder(p), View::SignalVec(svec)) => {
                svec.replace(parent, &p);
                Self::SignalVec(svec)
            }
            (Self::Node(n), View::Empty) => {
                let placeholder = create_empty_node();
                parent.replace_child(&placeholder, n.node()).unwrap();
                Self::Placeholder(placeholder)
            }
            (Self::Node(old), View::Node(new)) => {
                parent.replace_child(new.node(), old.node()).unwrap();
                Self::Node(new)
            }
            (Self::Node(n), View::Signal(sig)) => {
                sig.replace(parent, n.node());
                Self::Signal(sig)
            }
            (Self::Node(n), View::SignalVec(svec)) => {
                svec.replace(parent, n.node());
                Self::SignalVec(svec)
            }
            (Self::Signal(sig), new) => sig.replace_with(parent, new),
            (Self::SignalVec(svec), new) => svec.replace_with(parent, new),
            (Self::Fragment(_), _) => todo!(),
            (_, View::Fragment(_)) => todo!(),
        }
    }

    pub fn remove_from_parent(&self, parent: &web_sys::Node) {
        match self {
            RetainedView::Placeholder(p) => {
                parent.remove_child(&p).unwrap();
            }
            RetainedView::Node(n) => {
                parent.remove_child(&n.node()).unwrap();
            }
            RetainedView::Fragment(_f) => {
                todo!();
            }
            RetainedView::Signal(sig) => {
                sig.remove_from_parent(parent);
            }
            RetainedView::SignalVec(svec) => {
                svec.remove_from_parent(parent);
            }
        }
    }

    pub(crate) fn insert_before(&self, parent: &web_sys::Node, before: &web_sys::Node) {
        match self {
            RetainedView::Placeholder(p) => {
                parent.insert_before(p, Some(before)).unwrap();
            }
            RetainedView::Node(n) => {
                parent.insert_before(n.node(), Some(before)).unwrap();
            }
            RetainedView::Fragment(_) => todo!(),
            RetainedView::Signal(sig) => {
                sig.insert_before(parent, before);
            }
            RetainedView::SignalVec(svec) => {
                svec.insert_before(parent, before);
            }
        }
    }

    pub(crate) fn prepend_before_self(&self, parent: &web_sys::Node, new: &RetainedView) {
        match &self {
            RetainedView::Placeholder(p) => {
                new.insert_before(parent, p);
            }
            RetainedView::Node(n) => {
                new.insert_before(parent, n.node());
            }
            RetainedView::Fragment(_) => todo!(),
            RetainedView::Signal(sig) => {
                sig.prepend_before_self(parent, new);
            }
            RetainedView::SignalVec(svec) => {
                svec.prepend_before_self(parent, new);
            }
        }
    }

    // pub fn replace(&self, parent: &web_sys::Node, old_node: &web_sys::Node) {
    //     match self {
    //         RetainedView::Placeholder(p) => {
    //             parent.replace_child(&p, old_node).unwrap();
    //         }
    //         RetainedView::Node(n) => {
    //             parent.replace_child(n.node(), old_node).unwrap();
    //         }
    //         RetainedView::Fragment(_frag) => todo!(),
    //         RetainedView::Signal(sig) => sig.replace(parent, old_node),
    //     }
    // }

    pub fn as_placeholder(&self) -> Option<&web_sys::Node> {
        if let Self::Placeholder(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
