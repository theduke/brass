use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use futures::{
    future::{AbortHandle, Abortable},
    Future,
};
use futures_signals::{
    signal::{Mutable, Signal, SignalExt},
    signal_vec::{SignalVec, SignalVecExt, VecDiff},
};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use super::{Attr, DomEvent, Event, Style, Tag};
use crate::{
    component::{build_component, Component},
    web::{
        self, add_event_lister, create_element, create_empty_node, create_text, elem_add_class,
        elem_remove_class, elem_set_class_js, empty_string, remove_attr, set_attribute, set_style,
        set_text_data, DomStr,
    },
};

pub enum View {
    Empty,
    Node(Node),
    Fragment(Fragment),
}

impl From<Node> for View {
    fn from(n: Node) -> Self {
        Self::Node(n)
    }
}

impl From<TagBuilder> for View {
    fn from(t: TagBuilder) -> Self {
        Self::Node(t.build())
    }
}

impl From<Fragment> for View {
    fn from(f: Fragment) -> Self {
        Self::Fragment(f)
    }
}

impl View {
    pub(crate) fn attach(&self, parent: &web_sys::Element) {
        match self {
            View::Empty => {

            }
            View::Empty => {}
            View::Node(n) => {
                n.attach(parent);
            }
            View::Fragment(f) => {
                for item in &f.items {
                    item.attach(parent);
                }
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
}

pub struct Fragment {
    pub items: Vec<View>,
}

pub struct Node {
    pub node: web_sys::Node,
    after_remove: Vec<Box<dyn FnOnce()>>,
    aborts: Vec<AbortHandle>,
}

impl Node {
    pub(crate) fn attach(&self, parent: &web_sys::Element) {
        parent.append_child(&self.node).unwrap();
    }

    pub fn new_text(value: DomStr<'_>) -> Self {
        let text = web::create_text(value);
        Self {
            node: text.into(),
            after_remove: Vec::new(),
            aborts: Vec::new(),
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        for callback in self.after_remove.drain(..) {
            callback();
        }
        for abort in self.aborts.drain(..) {
            abort.abort();
        }
    }
}

pub struct TagBuilder<T = ()> {
    pub(crate) node: Node,
    _marker: PhantomData<T>,
}

impl TagBuilder<()> {
    pub fn new(tag: Tag) -> Self {
        // TODO: use cache!
        let elem = create_element(tag);
        Self {
            node: Node {
                node: elem.into(),
                after_remove: Vec::new(),
                aborts: Vec::new(),
            },
            _marker: PhantomData,
        }
    }

    pub fn elem(&self) -> &web_sys::Element {
        self.node.node.unchecked_ref()
    }

    pub fn register_future<F: Future<Output = ()> + 'static>(&mut self, f: F) {
        let (handle, reg) = AbortHandle::new_pair();
        let f = Abortable::new(f, reg);
        self.node.aborts.push(handle);
        // TODO: add a spawn_local_boxed method to wasm_bindgen_futures to
        // allow for a non-generic helper function that doesn't do double-boxing
        // (spawn_local boxes the future)
        spawn_local(async move {
            f.await.ok();
        });
    }

    pub fn add_after_remove<F: FnOnce() + 'static>(&mut self, f: F) {
        self.node.after_remove.push(Box::new(f));
    }

    #[inline]
    pub fn after_remove<F: FnOnce() + 'static>(mut self, f: F) -> Self {
        self.add_after_remove(f);
        self
    }

    pub fn add_bind<V: 'static>(&mut self, value: V) {
        self.add_after_remove(move || {
            std::mem::drop(value);
        });
    }

    pub fn bind<V: 'static>(mut self, value: V) -> Self {
        self.add_bind(value);
        self
    }

    pub fn add_attr<'a, I: Into<DomStr<'a>>>(&mut self, attr: Attr, value: I) {
        set_attribute(self.elem(), attr, value.into());
    }

    pub fn attr<'a, I: Into<DomStr<'a>>>(self, attr: Attr, value: I) -> Self {
        set_attribute(self.elem(), attr, value.into());
        self
    }

    pub fn style_raw<'a, I: Into<DomStr<'a>>>(self, value: I) -> Self {
        set_attribute(self.elem(), Attr::Style, value.into());
        self
    }

    pub fn set_style<'a, I: Into<DomStr<'a>>>(&mut self, style: Style, value: I) {
        set_style(self.elem(), style, value.into());
    }

    #[inline]
    pub fn style<'a, I: Into<DomStr<'a>>>(mut self, style: Style, value: I) -> Self {
        self.set_style(style, value);
        self
    }

    pub fn add_style_signal<V, S>(&mut self, style: Style, signal: S)
    where
        V: Into<DomStr<'static>>,
        S: Signal<Item = V> + 'static,
    {
        let elem = self.elem().clone();
        let f = signal.for_each(move |value| {
            set_style(&elem, style, value.into());
            async {}
        });
        self.register_future(f);
    }

    #[inline]
    pub fn style_signal<V, S>(mut self, style: Style, signal: S) -> Self
    where
        V: Into<DomStr<'static>>,
        S: Signal<Item = V> + 'static,
    {
        self.add_style_signal(style, signal);
        self
    }

    pub fn attr_toggle(self, attr: Attr) -> Self {
        self.attr(attr, empty_string())
    }

    pub fn attr_toggle_if(self, flag: bool, attr: Attr) -> Self {
        if flag {
            self.attr(attr, empty_string())
        } else {
            self
        }
    }

    pub fn add_attr_signal<V, S>(&mut self, attr: Attr, signal: S)
    where
        V: Into<DomStr<'static>>,
        S: Signal<Item = V> + 'static,
    {
        let elem = self.elem().clone();
        let f = signal.for_each(move |value| {
            set_attribute(&elem, attr, value.into());
            async {}
        });
        self.register_future(f);
    }

    pub fn add_attr_signal_opt<V, S>(&mut self, attr: Attr, signal: S)
    where
        V: Into<DomStr<'static>>,
        S: Signal<Item = Option<V>> + 'static,
    {
        let elem = self.elem().clone();
        let mut is_added = false;
        let f = signal.for_each(move |opt| {
            if let Some(value) = opt {
                set_attribute(&elem, attr, value.into());
                is_added = true;
            } else if is_added {
                remove_attr(&elem, attr);
            }
            async {}
        });
        self.register_future(f);
    }

    #[inline]
    pub fn attr_signal<V, S>(mut self, attr: Attr, signal: S) -> Self
    where
        V: Into<DomStr<'static>>,
        S: Signal<Item = V> + 'static,
    {
        self.add_attr_signal(attr, signal);
        self
    }

    pub fn add_attr_signal_toggle<S>(&mut self, attr: Attr, signal: S)
    where
        S: Signal<Item = bool> + 'static,
    {
        let elem = self.elem().clone();
        let f = signal.for_each(move |flag| {
            if flag {
                // TODO: use cached empty string.
                set_attribute(&elem, attr, empty_string().into());
            } else {
                remove_attr(&elem, attr);
            }
            async {}
        });
        self.register_future(f);
    }

    #[inline]
    pub fn attr_signal_toggle<S>(mut self, attr: Attr, signal: S) -> Self
    where
        S: Signal<Item = bool> + 'static,
    {
        self.add_attr_signal_toggle(attr, signal);
        self
    }

    pub fn add_class<'a, I>(&mut self, class: I)
    where
        I: Into<DomStr<'a>>,
    {
        let value = class.into();
        elem_add_class(self.elem(), &value);
    }

    #[inline]
    pub fn class<'a, I>(mut self, class: I) -> Self
    where
        I: Into<DomStr<'a>>,
    {
        self.add_class(class);
        self
    }

    pub fn add_classes_raw(&mut self, classes: &str) {
        for cls in classes.split(" ") {
            self.add_class(cls);
        }
    }

    pub fn classes_raw(mut self, classes: &str) -> Self {
        self.add_classes_raw(classes);
        self
    }

    pub fn class_if<'a, I>(mut self, flag: bool, class: I) -> Self
    where
        I: Into<DomStr<'a>>,
    {
        if flag {
            self.add_class(class);
        }
        self
    }

    pub fn add_class_opt<'a, I>(&mut self, class: Option<I>)
    where
        I: Into<DomStr<'a>>,
    {
        if let Some(class) = class {
            elem_add_class(self.elem(), &class.into());
        }
    }

    #[inline]
    pub fn class_opt<'a, I>(mut self, class: Option<I>) -> Self
    where
        I: Into<DomStr<'a>>,
    {
        self.add_class_opt(class);
        self
    }

    pub fn add_classes<'a, S, I>(&mut self, iter: I)
    where
        S: Into<DomStr<'a>>,
        I: IntoIterator<Item = S> + 'a,
    {
        for class in iter {
            self.add_class(class);
        }
    }

    #[inline]
    pub fn classes<'a, S, I>(mut self, iter: I) -> Self
    where
        S: Into<DomStr<'a>>,
        I: IntoIterator<Item = S> + 'a,
    {
        {
            self.add_classes(iter);
        }

        self
    }

    pub fn add_class_signal<I, S>(&mut self, signal: S)
    where
        I: Into<DomStr<'static>>,
        S: Signal<Item = I> + 'static,
    {
        let elem = self.elem().clone();
        let mut current = None;
        self.register_future(signal.for_each(move |value| {
            if let Some(current) = current.take() {
                elem_remove_class(&elem, &current);
            }
            let class = value.into();
            elem_add_class(&elem, &class);
            current = Some(class);
            async {}
        }));
    }

    #[inline]
    pub fn class_signal<I, S>(mut self, signal: S) -> Self
    where
        I: Into<DomStr<'static>>,
        S: Signal<Item = I> + 'static,
    {
        self.add_class_signal(signal);
        self
    }

    pub fn add_class_signal_toggle<I, S>(&mut self, class: I, signal: S)
    where
        I: Into<DomStr<'static>>,
        S: Signal<Item = bool> + 'static,
    {
        let class = class.into();

        let elem = self.elem().clone();
        let mut is_added = false;
        self.register_future(signal.for_each(move |flag| {
            if flag {
                if !is_added {
                    elem_add_class(&elem, &class);
                    is_added = true;
                }
            } else {
                if is_added {
                    elem_remove_class(&elem, &class);
                    is_added = false;
                }
            }
            async {}
        }));
    }

    #[inline]
    pub fn class_signal_toggle<I, S>(mut self, class: I, signal: S) -> Self
    where
        I: Into<DomStr<'static>>,
        S: Signal<Item = bool> + 'static,
    {
        self.add_class_signal_toggle(class, signal);
        self
    }

    #[inline]
    pub fn add_classes_signal<'a, V, S>(&mut self, signal: S)
    where
        V: Into<DomStr<'static>>,
        S: SignalVec<Item = V> + 'static,
    {
        // TODO: we really want a custom ClassList signal implementation instead of
        // MutableVec.

        let elem = self.elem().clone();

        self.register_future(signal.for_each(move |diff| {
            match diff {
                VecDiff::Replace { values } => {
                    elem_set_class_js(&elem, empty_string());
                    for value in values {
                        elem_add_class(&elem, &value.into());
                    }
                }
                VecDiff::InsertAt { index: _, value } => {
                    elem_add_class(&elem, &value.into());
                }
                VecDiff::UpdateAt { index: _, value: _ } => {
                    unimplemented!()
                }
                VecDiff::RemoveAt { index: _ } => {
                    unimplemented!()
                }
                VecDiff::Move {
                    old_index: _,
                    new_index: _,
                } => {}
                VecDiff::Push { value } => {
                    elem_add_class(&elem, &value.into());
                }
                VecDiff::Pop {} => {
                    unimplemented!()
                }
                VecDiff::Clear {} => {
                    elem_set_class_js(&elem, empty_string());
                }
            }
            async {}
        }));
    }

    pub fn add_child_text<'a>(&mut self, value: DomStr<'a>) {
        let text = create_text(value);
        self.node.node.append_child(&text).unwrap();
    }

    #[inline]
    pub fn child_text<'a, S>(mut self, value: S) -> Self
    where
        S: Into<DomStr<'a>>,
    {
        self.add_child_text(value.into());
        self
    }

    pub fn add_child_text_signal<V, S>(&mut self, signal: S)
    where
        V: Into<DomStr<'static>>,
        S: Signal<Item = V> + 'static,
    {
        let text = create_text("".into());
        self.node.node.append_child(&text).unwrap();

        let f = signal.for_each(move |value| {
            set_text_data(&text, &value.into());
            async {}
        });
        self.register_future(f);
    }

    #[inline]
    pub fn child_text_signal<V, S>(mut self, signal: S) -> Self
    where
        V: Into<DomStr<'static>>,
        S: Signal<Item = V> + 'static,
    {
        self.add_child_text_signal(signal);
        self
    }

    fn add_node(&mut self, mut node: Node) {
        // TODO use custom binding for efficiency?
        self.node.node.append_child(&node.node).unwrap();
        self.node.after_remove.extend(node.after_remove.drain(..));
        self.node.aborts.extend(node.aborts.drain(..));
    }

    #[inline]
    pub fn add_child(&mut self, child: TagBuilder) {
        self.add_node(child.node);
    }

    pub fn child(mut self, child: TagBuilder) -> Self {
        self.add_child(child);
        self
    }

    pub fn add_child_signal<T, S>(&mut self, signal: S)
    where
        T: Into<Option<TagBuilder>>,
        S: Signal<Item = T> + 'static,
    {
        let parent = self.elem().clone();
        // TODO: use cache!
        // TODO: use something other than a span? maybe a comment node?
        let marker = create_empty_node();
        // TODO use custom binding for efficiency?
        parent.append_child(&marker).unwrap();

        let mut current_node: Option<Node> = None;
        let f = signal.for_each(move |opt| {
            if let Some(tag) = opt.into() {
                if let Some(old) = current_node.take() {
                    parent.replace_child(&tag.node.node, &old.node).unwrap();
                } else {
                    parent.replace_child(&tag.node.node, &marker).unwrap();
                }
                current_node = Some(tag.node);
            } else {
                if let Some(current) = current_node.take() {
                    parent.replace_child(&marker, &current.node).unwrap();
                }
            }

            async {}
        });
        self.register_future(f);
    }

    #[inline]
    pub fn child_signal<S>(mut self, signal: S) -> Self
    where
        S: Signal<Item = TagBuilder> + 'static,
    {
        self.add_child_signal(signal);
        self
    }

    #[inline]
    pub fn child_signal_opt<S>(mut self, signal: S) -> Self
    where
        S: Signal<Item = Option<TagBuilder>> + 'static,
    {
        self.add_child_signal(signal);
        self
    }

    pub fn add_children_signal<V, S, R>(
        &mut self,
        signal: S,
        render: R,
        fallback: Option<TagBuilder>,
    ) where
        S: SignalVec<Item = V> + 'static,
        R: Fn(&V) -> Node + 'static,
    {
        let parent = self.elem().clone();
        // TODO: use cache!
        // TODO: use something other than a span? maybe a comment node?
        let marker = create_empty_node();

        let mut fallback_visible = if let Some(e) = &fallback {
            parent.append_child(e.elem()).unwrap();
            true
        } else {
            false
        };

        // TODO use custom binding for efficiency?

        parent.append_child(&marker).unwrap();

        let mut children = Vec::<Node>::new();

        let f = signal.for_each(move |patch| {
            match patch {
                VecDiff::Replace { values } => {
                    tracing::warn!("VecDiff::Replace {}", values.len());
                    children.drain(..).for_each(|child| {
                        parent.remove_child(&child.node).unwrap();
                    });

                    if values.is_empty() && !fallback_visible {
                        if let Some(e) = fallback.as_ref() {
                            parent.insert_before(e.elem(), Some(&marker)).unwrap();
                            fallback_visible = true;
                        }
                    } else {
                        if fallback_visible {
                            parent
                                .remove_child(fallback.as_ref().unwrap().elem())
                                .unwrap();
                            fallback_visible = false;
                        }
                        for value in values {
                            let child = render(&value);
                            parent.insert_before(&child.node, Some(&marker)).unwrap();
                            children.push(child);
                        }
                    }
                }
                VecDiff::InsertAt { index, value } => {
                    tracing::warn!("VecDiff::InsertAt {}", index);
                    if fallback_visible {
                        parent
                            .remove_child(fallback.as_ref().unwrap().elem())
                            .unwrap();
                        fallback_visible = false;
                    }

                    let child = render(&value);
                    if index == 0 {
                        parent.prepend_with_node_1(&child.node).unwrap();
                    }
                    children.insert(0, child);
                }
                VecDiff::UpdateAt { index: _, value: _ } => {
                    todo!()
                }
                VecDiff::RemoveAt { index } => {
                    tracing::warn!("VecDiff::RemoveAt {}", index);
                    let removed = children.remove(index);
                    parent.remove_child(&removed.node).unwrap();

                    if children.is_empty() && !fallback_visible {
                        if let Some(e) = fallback.as_ref() {
                            parent.insert_before(e.elem(), Some(&marker)).unwrap();
                            fallback_visible = true;
                        }
                    }
                }
                VecDiff::Move {
                    old_index: _,
                    new_index: _,
                } => todo!(),
                VecDiff::Push { value } => {
                    tracing::warn!("VecDiff::Push");
                    let child = render(&value);
                    parent.insert_before(&child.node, Some(&marker)).unwrap();
                    children.push(child);

                    if fallback_visible {
                        parent
                            .remove_child(fallback.as_ref().unwrap().elem())
                            .unwrap();
                        fallback_visible = false;
                    }
                }
                VecDiff::Pop {} => {
                    tracing::warn!("VecDiff::Pop");
                    if let Some(old) = children.pop() {
                        parent.remove_child(&old.node).unwrap();
                    }

                    if children.is_empty() && !fallback_visible {
                        if let Some(e) = fallback.as_ref() {
                            parent.insert_before(e.elem(), Some(&marker)).unwrap();
                            fallback_visible = true;
                        }
                    }
                }
                VecDiff::Clear {} => {
                    tracing::warn!("VecDiff::Clear");
                    children.drain(..).for_each(|child| {
                        parent.remove_child(&child.node).unwrap();
                    });

                    if !fallback_visible {
                        if let Some(e) = fallback.as_ref() {
                            parent.insert_before(e.elem(), Some(&marker)).unwrap();
                            fallback_visible = true;
                        }
                    }
                }
            }
            async {}
        });

        self.register_future(f);
    }

    pub fn children_signal<V, S, R>(mut self, signal: S, render: R) -> Self
    where
        S: SignalVec<Item = V> + 'static,
        R: Fn(&V) -> Node + 'static,
    {
        self.add_children_signal(signal, render, None);
        self
    }

    pub fn children_signal_with_fallback<V, S, R>(
        mut self,
        signal: S,
        render: R,
        fallback: TagBuilder,
    ) -> Self
    where
        S: SignalVec<Item = V> + 'static,
        R: Fn(&V) -> Node + 'static,
    {
        self.add_children_signal(signal, render, Some(fallback));
        self
    }

    pub fn add_event_listener<F>(&mut self, event: Event, mut handler: F)
    where
        F: FnMut(web_sys::Event) + 'static,
    {
        // TODO: use global event handler.
        // TODO: use callback cache.
        let callback =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::Event| {
                handler(event);
            }) as Box<dyn FnMut(web_sys::Event)>);

        add_event_lister(self.elem(), event, callback.as_ref().unchecked_ref());
        self.node.after_remove.push(Box::new(move || {
            std::mem::drop(callback);
        }));
    }

    pub fn add_event_listener_cast<E, F>(&mut self, event: Event, mut handler: F)
    where
        E: AsRef<web_sys::Event> + JsCast,
        F: FnMut(E) + 'static,
    {
        // TODO: use global event handler.
        // TODO: use callback cache.
        let callback =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |raw_event: web_sys::Event| {
                match raw_event.dyn_into::<E>() {
                    Ok(event) => {
                        handler(event);
                    }
                    Err(err) => {
                        panic!(
                        "Event handler received invalid invalid event type (Expected {}) - {:?}",
                        std::any::type_name::<E>(),
                        err
                    );
                    }
                }
            }) as Box<dyn FnMut(web_sys::Event)>);

        add_event_lister(self.elem(), event, callback.as_ref().unchecked_ref());
        self.node.after_remove.push(Box::new(move || {
            std::mem::drop(callback);
        }));
    }

    pub fn add_dom_event_listener<E, F>(&mut self, mut handler: F)
    where
        E: DomEvent,
        F: FnMut(E) + 'static,
    {
        self.add_event_listener(E::event_type(), move |raw_event| {
            if let Some(event) = E::from_dom(raw_event) {
                handler(event);
            }
        });
    }

    pub fn on<E, F>(mut self, handler: F) -> Self
    where
        E: DomEvent,
        F: FnMut(E) + 'static,
    {
        self.add_dom_event_listener(handler);
        self
    }

    pub fn on_event<F>(mut self, event: Event, handler: F) -> Self
    where
        F: Fn(web_sys::Event) + 'static,
    {
        self.add_event_listener(event, handler);
        self
    }

    pub fn add_component<C: Component>(&mut self, props: C::Properties) {
        let tag = build_component::<C>(props);
        self.add_bind(tag);
    }

    #[inline]
    pub fn component<C: Component>(mut self, props: C::Properties) -> Self {
        self.add_component::<C>(props);
        self
    }

    pub fn and<A: Apply>(mut self, apply: A) -> Self {
        apply.apply(&mut self);
        self
    }

    pub fn add_iter<A: Apply, I: IntoIterator<Item = A>>(&mut self, iter: I) {
        for item in iter {
            item.apply(self);
        }
    }

    #[inline]
    pub fn and_iter<A: Apply, I: IntoIterator<Item = A>>(mut self, iter: I) -> Self {
        self.add_iter(iter);
        self
    }

    #[inline]
    pub fn build(self) -> Node {
        self.node
    }

    #[inline]
    pub fn into_view(self) -> View {
        View::Node(self.build())
    }
}

pub trait Render {
    fn render(self) -> View;
}

impl<R: Render> Apply for R {
    fn apply(self, tag: &mut TagBuilder) {
        self.render().apply(tag);
    }
}

pub trait Apply {
    fn apply(self, tag: &mut TagBuilder);
}

impl<'a> Apply for &'a str {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_child_text(self.into());
    }
}

impl<'a> Apply for &'a String {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_child_text(self.into());
    }
}

impl Apply for String {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_child_text(self.into());
    }
}

impl<'a> Apply for DomStr<'a> {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_child_text(self);
    }
}

impl Apply for TagBuilder {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_child(self);
    }
}

impl Apply for Node {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_node(self);
    }
}

impl Apply for View {
    fn apply(self, tag: &mut TagBuilder) {
        match self {
            View::Empty => {}
            View::Node(n) => tag.add_node(n),
            View::Fragment(Fragment { items }) => {
                for item in items {
                    item.apply(tag);
                }
            }
        }
    }
}

impl Apply for Fragment {
    fn apply(self, tag: &mut TagBuilder) {
        for item in self.items {
            item.apply(tag);
        }
    }
}

impl<'a> Apply for &'a Mutable<String> {
    fn apply(self, tag: &mut TagBuilder) {
        // TODO: possible to avoid cloning?
        tag.add_child_text_signal(self.signal_cloned());
    }
}

pub struct TextSignal<S>(pub S);

impl<S, O> Apply for TextSignal<S>
where
    S: Signal<Item = O> + 'static,
    O: Into<DomStr<'static>>,
{
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_child_text_signal(self.0)
    }
}

impl<I> Apply for Option<I>
where
    I: Apply,
{
    fn apply(self, tag: &mut TagBuilder) {
        if let Some(inner) = self {
            inner.apply(tag);
        }
    }
}

// impl <A, I> Apply for I where A: Apply, I: IntoIterator<Item = A> {
//     fn apply(self, tag: &mut TagBuilder) {
//         for item in self {
//             item.apply(tag);
//         }
//     }
// }

pub struct WithSignal<S>(pub S);

impl<S, I> Apply for WithSignal<S>
where
    S: Signal<Item = I> + 'static,
    I: Into<Option<TagBuilder>>,
{
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_child_signal(self.0);
    }
}

pub struct ApplyFuture<F>(pub F);

impl<F> Apply for ApplyFuture<F>
where
    F: std::future::Future<Output = TagBuilder> + 'static,
{
    fn apply(self, tag: &mut TagBuilder) {
        let mut wrapper = builder::div();
        let elem = wrapper.elem().clone();
        let keeper: Rc<RefCell<Option<TagBuilder>>> = std::rc::Rc::new(RefCell::new(None));
        wrapper.add_bind(keeper.clone());

        let f = self.0;
        wrapper.register_future(async move {
            let child = f.await;
            elem.append_child(child.elem()).ok();
            *keeper.borrow_mut() = Some(child);
        });

        tag.add_child(wrapper);
    }
}

// impl<S> Apply for SignalOpt<S>
//     where S: Signal<Item = >
// {

// }

impl<A1: Apply, A2: Apply> Apply for (A1, A2) {
    fn apply(self, tag: &mut TagBuilder) {
        self.0.apply(tag);
        self.1.apply(tag);
    }
}

impl<A1: Apply, A2: Apply, A3: Apply> Apply for (A1, A2, A3) {
    fn apply(self, tag: &mut TagBuilder) {
        self.0.apply(tag);
        self.1.apply(tag);
        self.2.apply(tag);
    }
}

impl<A1: Apply, A2: Apply, A3: Apply, A4: Apply> Apply for (A1, A2, A3, A4) {
    fn apply(self, tag: &mut TagBuilder) {
        self.0.apply(tag);
        self.1.apply(tag);
        self.2.apply(tag);
        self.3.apply(tag);
    }
}

impl<A1: Apply, A2: Apply, A3: Apply, A4: Apply, A5: Apply> Apply for (A1, A2, A3, A4, A5) {
    fn apply(self, tag: &mut TagBuilder) {
        self.0.apply(tag);
        self.1.apply(tag);
        self.2.apply(tag);
        self.3.apply(tag);
        self.4.apply(tag);
    }
}

impl<A1: Apply, A2: Apply, A3: Apply, A4: Apply, A5: Apply, A6: Apply> Apply
    for (A1, A2, A3, A4, A5, A6)
{
    fn apply(self, tag: &mut TagBuilder) {
        self.0.apply(tag);
        self.1.apply(tag);
        self.2.apply(tag);
        self.3.apply(tag);
        self.4.apply(tag);
        self.5.apply(tag);
    }
}

pub trait AttrValueApply<M> {
    fn attr_apply(self, attr: Attr, b: &mut TagBuilder);
}

impl<V: Into<DomStr<'static>>> AttrValueApply<DomStr<'static>> for V {
    fn attr_apply(self, attr: Attr, b: &mut TagBuilder) {
        b.add_attr(attr, self)
    }
}

impl<V: Into<DomStr<'static>>, S: Signal<Item = V> + 'static> AttrValueApply<(S, DomStr<'static>)>
    for S
{
    fn attr_apply(self, attr: Attr, b: &mut TagBuilder) {
        b.add_attr_signal(attr, self)
    }
}

impl<V: Into<DomStr<'static>>, S: Signal<Item = Option<V>> + 'static>
    AttrValueApply<(S, Option<DomStr<'static>>)> for S
{
    fn attr_apply(self, attr: Attr, b: &mut TagBuilder) {
        b.add_attr_signal_opt(attr, self)
    }
}

pub trait EventHandlerApply<V> {
    fn event_handler_apply(self, event: Event, target: &mut TagBuilder);
}

impl<F> EventHandlerApply<fn()> for F
where
    F: FnMut() + 'static,
{
    fn event_handler_apply(mut self, event: Event, target: &mut TagBuilder) {
        target.add_event_listener(event, move |_| self())
    }
}

impl<F, E> EventHandlerApply<fn(E)> for F
where
    F: FnMut(E) + 'static,
    E: AsRef<web_sys::Event> + JsCast,
{
    fn event_handler_apply(self, event: Event, target: &mut TagBuilder) {
        target.add_event_listener_cast(event, self)
    }
}

pub mod builder {
    use super::{Tag, TagBuilder};

    #[inline]
    pub fn tag(tag: Tag) -> TagBuilder {
        TagBuilder::new(tag)
    }

    #[inline]
    pub fn div() -> TagBuilder {
        TagBuilder::new(Tag::Div)
    }

    #[inline]
    pub fn span() -> TagBuilder {
        TagBuilder::new(Tag::Span)
    }

    #[inline]
    pub fn button() -> TagBuilder {
        TagBuilder::new(Tag::Button)
    }

    #[inline]
    pub fn input() -> TagBuilder {
        TagBuilder::new(Tag::Input)
    }

    #[inline]
    pub fn p() -> TagBuilder {
        TagBuilder::new(Tag::P)
    }
}
