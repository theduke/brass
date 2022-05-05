use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use futures::Future;
use futures_signals::{
    signal::{Mutable, Signal, SignalExt},
    signal_vec::{SignalVec, SignalVecExt, VecDiff},
};
use js_sys::JsString;
use wasm_bindgen::JsCast;

use crate::{
    component::{build_component, Component},
    context::AppContext,
    web::{
        self, add_event_lister, create_element, create_text, elem_add_class, elem_remove_class,
        elem_set_class_js, empty_string, remove_attr, set_attribute, set_style, set_text_data,
        DomStr,
    },
};

use super::{
    signal_vec_view::SignalVecView, signal_view::SignalView, view::RetainedView, AbortGuard, Attr,
    DomEvent, Ev, Style, Tag, View,
};

pub struct Fragment {
    pub items: Vec<View>,
}

impl From<Fragment> for View {
    fn from(f: Fragment) -> Self {
        Self::Fragment(f)
    }
}

pub struct Node {
    node: web_sys::Node,
    after_remove: Vec<Box<dyn FnOnce()>>,
    aborts: Vec<AbortGuard>,

    children: Vec<RetainedView>,
}

impl Node {
    #[inline]
    pub fn node(&self) -> &web_sys::Node {
        &self.node
    }

    pub(crate) fn attach(&self, parent: &web_sys::Element) {
        parent.append_child(&self.node).unwrap();
    }

    pub fn new_text(value: DomStr<'_>) -> Self {
        let text = web::create_text(value);
        Self {
            node: text.into(),
            after_remove: Vec::new(),
            aborts: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        for callback in self.after_remove.drain(..) {
            callback();
        }
    }
}

impl From<Node> for View {
    fn from(n: Node) -> Self {
        Self::Node(n)
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
                children: Vec::new(),
            },
            _marker: PhantomData,
        }
    }

    #[doc(hidden)]
    pub fn from_node(node: Node) -> Self {
        Self {
            node,
            _marker: PhantomData,
        }
    }

    pub fn elem(&self) -> &web_sys::Element {
        self.node.node.unchecked_ref()
    }

    pub fn register_future<F: Future<Output = ()> + 'static>(&mut self, f: F) {
        let guard = AppContext::spawn_abortable(f);
        self.node.aborts.push(guard);
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

    // Attributes.

    pub fn add_attr<'a, I: Into<DomStr<'a>>>(&mut self, attr: Attr, value: I) {
        set_attribute(self.elem(), attr, value.into());
    }

    pub fn attr<'a, I: Into<DomStr<'a>>>(self, attr: Attr, value: I) -> Self {
        set_attribute(self.elem(), attr, value.into());
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

    // Class.

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

    // Style.

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

    pub fn add_class<'a, I>(&mut self, class: I)
    where
        I: Into<DomStr<'a>>,
    {
        let value = class.into();
        elem_add_class(self.elem(), &value);
    }

    pub fn add_text<'a>(&mut self, value: DomStr<'a>) {
        let text = create_text(value);
        self.node.node.append_child(&text).unwrap();
    }

    #[inline]
    pub fn text<'a, S>(mut self, value: S) -> Self
    where
        S: Into<DomStr<'a>>,
    {
        self.add_text(value.into());
        self
    }

    pub fn add_text_signal<V, S>(&mut self, signal: S)
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
    pub fn text_signal<V, S>(mut self, signal: S) -> Self
    where
        V: Into<DomStr<'static>>,
        S: Signal<Item = V> + 'static,
    {
        self.add_text_signal(signal);
        self
    }

    // Events.

    pub fn add_event_listener<F>(&mut self, event: Ev, mut handler: F)
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

    pub fn add_event_listener_cast<E, F>(&mut self, event: Ev, mut handler: F)
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

    pub fn on_event<F>(mut self, event: Ev, handler: F) -> Self
    where
        F: Fn(web_sys::Event) + 'static,
    {
        self.add_event_listener(event, handler);
        self
    }

    // Node.

    pub fn add_node(&mut self, node: Node) {
        // TODO use custom binding for efficiency?
        self.node.node.append_child(&node.node).unwrap();

        // TODO: only keep nodes that have event handlers, otherwise add fold
        // them into the current node with code below.
        // self.node.after_remove.extend(node.after_remove.drain(..));
        // self.node.aborts.extend(node.aborts.drain(..));

        self.node.children.push(RetainedView::Node(node));
    }

    #[inline]
    pub fn add_tag(&mut self, child: TagBuilder) {
        self.add_node(child.node);
    }

    pub fn tag(mut self, child: TagBuilder) -> Self {
        self.add_tag(child);
        self
    }

    // Signal.

    pub fn add_signal_view(&mut self, sig: SignalView) {
        sig.attach(self.node.node());
    }

    pub fn add_signal<T, S>(&mut self, signal: S)
    where
        T: Into<View>,
        S: Signal<Item = T> + 'static,
    {
        // TODO: add single-method constructor that already receives the parent?
        let sig = SignalView::new(signal);
        sig.attach(self.node.node());
        self.node.children.push(RetainedView::Signal(sig));
    }

    #[inline]
    pub fn signal<V, S>(mut self, signal: S) -> Self
    where
        V: Into<View>,
        S: Signal<Item = V> + 'static,
    {
        self.add_signal(signal);
        self
    }

    // SignalVec.

    pub fn add_signal_vec_view(&mut self, view: SignalVecView) {
        view.attach(self.node.node());
        self.node.children.push(RetainedView::SignalVec(view));
    }

    pub fn add_signal_vec<T, S, O, R>(&mut self, signal: S, render: R, fallback: Option<View>)
    where
        S: SignalVec<Item = T> + 'static,
        R: Fn(&T) -> O + 'static,
        O: Render,
    {
        self.add_signal_vec_view(SignalVecView::new(signal, render, fallback))
    }

    pub fn signal_vec<T, S, O, R>(mut self, signal: S, render: R) -> Self
    where
        S: SignalVec<Item = T> + 'static,
        R: Fn(&T) -> O + 'static,
        O: Render,
    {
        self.add_signal_vec(signal, render, None);
        self
    }

    pub fn signal_vec_with_fallback<T, S, O, R, F>(
        mut self,
        signal: S,
        render: R,
        fallback: F,
    ) -> Self
    where
        S: SignalVec<Item = T> + 'static,
        R: Fn(&T) -> O + 'static,
        O: Render,
        F: Render,
    {
        self.add_signal_vec(signal, render, Some(fallback.render()));
        self
    }

    // Component.

    pub fn add_component<C: Component>(&mut self, props: C::Properties) {
        let tag = build_component::<C>(props);
        self.add_bind(tag);
    }

    #[inline]
    pub fn component<C: Component>(mut self, props: C::Properties) -> Self {
        self.add_component::<C>(props);
        self
    }

    pub fn add_view(&mut self, view: View) {
        match view {
            View::Empty => {}
            View::Node(n) => {
                n.attach(self.elem());
                self.node.children.push(RetainedView::Node(n));
            }
            View::Fragment(_) => todo!(),
            View::Signal(s) => {
                self.add_signal_view(s);
            }
            View::SignalVec(s) => {
                self.add_signal_vec_view(s);
            }
        }
    }

    pub fn and<A: Apply>(mut self, item: A) -> Self {
        item.apply(&mut self);
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

impl From<TagBuilder> for View {
    fn from(t: TagBuilder) -> Self {
        Self::Node(t.build())
    }
}

pub trait Render {
    fn render(self) -> View;
}

impl<R: Render> Apply for R {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_view(self.render());
    }
}

pub trait Apply {
    fn apply(self, tag: &mut TagBuilder);
}

impl<'a> Apply for &'a str {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_text(self.into());
    }
}

impl<'a> Apply for &'a JsString {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_text(self.into());
    }
}

impl Apply for JsString {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_text(self.into());
    }
}

impl<'a> Apply for &'a String {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_text(self.into());
    }
}

impl Apply for String {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_text(self.into());
    }
}

impl<'a> Apply for DomStr<'a> {
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_text(self);
    }
}

impl Render for Node {
    fn render(self) -> View {
        View::Node(self)
    }
}

impl Render for TagBuilder {
    fn render(self) -> View {
        self.node.render()
    }
}

impl Render for View {
    fn render(self) -> View {
        self
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
        tag.add_text_signal(self.signal_cloned());
    }
}

pub struct TextSignal<S>(pub S);

impl<S, O> Apply for TextSignal<S>
where
    S: Signal<Item = O> + 'static,
    O: Into<DomStr<'static>>,
{
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_text_signal(self.0)
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
    I: Into<View>,
{
    fn apply(self, tag: &mut TagBuilder) {
        tag.add_signal(self.0);
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

        tag.add_tag(wrapper);
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
    fn event_handler_apply(self, event: Ev, target: &mut TagBuilder);
}

impl<F> EventHandlerApply<fn()> for F
where
    F: FnMut() + 'static,
{
    fn event_handler_apply(mut self, event: Ev, target: &mut TagBuilder) {
        target.add_event_listener(event, move |_| self())
    }
}

impl<F, E> EventHandlerApply<fn(E)> for F
where
    F: FnMut(E) + 'static,
    E: AsRef<web_sys::Event> + JsCast,
{
    fn event_handler_apply(self, event: Ev, target: &mut TagBuilder) {
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
