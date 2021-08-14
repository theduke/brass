use crate::{
    app::EventCallbackId,
    dom::{Attr, Event, Tag},
    Component, Shared, Str,
};

use super::{EventCallback, EventHandler, Ref, VComponent, VNode, VRef, VTag, VText};

pub fn component<C: Component>(props: C::Properties) -> VNode {
    VNode::Component(VComponent::new::<C>(props))
}

pub struct TagBuilder {
    tag: VTag,
}

impl TagBuilder {
    pub fn new(tag: Tag) -> Self {
        Self {
            tag: VTag::new(tag),
        }
    }

    pub fn new_tag(tag: VTag) -> Self {
        Self { tag }
    }

    pub fn wrap<P: Into<TagBuilder>>(self, parent: P) -> TagBuilder {
        parent.into().and(self)
    }

    pub fn add_attr(&mut self, attr: Attr, value: impl Into<Str>) {
        self.tag.attributes.insert(attr.into(), value.into());
    }

    pub fn attr(mut self, attribute: Attr, value: impl Into<Str>) -> Self {
        self.tag.attributes.insert(attribute.into(), value.into());
        self
    }

    pub fn attr_if(mut self, flag: bool, attribute: Attr, value: impl Into<Str>) -> Self {
        if flag {
            self.tag.attributes.insert(attribute.into(), value.into());
        }
        self
    }

    pub fn attr_toggle(mut self, attribute: Attr) -> Self {
        self.tag.attributes.insert(attribute.into(), Str::empty());
        self
    }

    pub fn attr_toggle_if(mut self, flag: bool, attribute: Attr) -> Self {
        if flag {
            self.tag.attributes.insert(attribute.into(), Str::empty());
        }
        self
    }

    #[inline]
    pub fn class(self, cls: impl Into<Str>) -> Self {
        self.attr(Attr::Class, cls)
    }

    #[inline]
    pub fn class_if(self, flag: bool, cls: &str) -> Self {
        if flag {
            self.attr(Attr::Class, cls)
        } else {
            self
        }
    }

    pub fn class_opt(self, cls: Option<impl Into<Str>>) -> Self {
        if let Some(cls) = cls {
            self.attr(Attr::Class, cls)
        } else {
            self
        }
    }

    pub fn and_class(mut self, class: &str) -> Self {
        if let Some(old) = self.tag.attributes.get_mut(&Attr::Class) {
            let value = std::mem::take(old);
            *old = value.push(' ').push_str(class);
            self
        } else {
            self.class(class)
        }
    }

    pub fn and_class_if(mut self, flag: bool, class: &str) -> Self {
        if flag {
            if let Some(old) = self.tag.attributes.get_mut(&Attr::Class) {
                let value = std::mem::take(old);
                *old = value.push(' ').push_str(class);
                self
            } else {
                self.class(class)
            }
        } else {
            self
        }
    }

    #[inline]
    pub fn style_raw(self, style: impl Into<Str>) -> Self {
        self.attr(Attr::Style, style)
    }

    pub fn attr_opt(mut self, attribute: Attr, value: Option<impl Into<Str>>) -> Self {
        if let Some(v) = value {
            self.tag.attributes.insert(attribute.into(), v.into());
        }
        self
    }

    #[inline]
    pub fn and(mut self, child: impl DomExtend) -> Self {
        child.extend(&mut self);
        self
    }

    #[inline]
    pub fn and_opt(mut self, child: Option<impl DomExtend>) -> Self {
        if let Some(child) = child {
            child.extend(&mut self);
        }
        self
    }

    pub fn and_iter<T, I>(mut self, iter: I) -> Self
    where
        T: DomExtend,
        I: IntoIterator<Item = T>,
    {
        for item in iter.into_iter() {
            item.extend(&mut self);
        }

        self
    }

    pub fn and_if<F, T>(mut self, flag: bool, f: F) -> Self
    where
        T: DomExtend,
        F: FnOnce() -> T,
    {
        if flag {
            let item = f();
            item.extend(&mut self);
        }
        self
    }

    pub fn and_iter_if<T, I, F>(mut self, flag: bool, f: F) -> Self
    where
        T: DomExtend,
        I: IntoIterator<Item = T>,
        F: FnOnce() -> I,
    {
        if flag {
            let iter = f();
            for item in iter.into_iter() {
                item.extend(&mut self);
            }
        }

        self
    }

    pub fn add_child(&mut self, child: impl Into<VNode>) {
        self.tag.children.push(child.into());
    }

    pub fn build(self) -> VNode {
        VNode::Tag(self.tag)
    }

    pub fn on(mut self, event: Event, callback: EventCallback) -> Self {
        self.tag.event_handlers.push(EventHandler {
            event,
            callback,
            callback_id: EventCallbackId::new_null(),
        });
        self
    }

    pub fn on_click(self, callback: EventCallback) -> Self {
        self.on(Event::Click, callback)
    }

    pub fn build_ref(self, vref: &Ref) -> VNode {
        VNode::Ref(VRef {
            tag: self.tag,
            data: vref.clone(),
        })
    }
}

impl From<TagBuilder> for VNode {
    fn from(b: TagBuilder) -> Self {
        b.build()
    }
}

pub trait Render {
    fn render(self) -> VNode;
}

pub trait RenderRef {
    fn render_ref(&self) -> VNode;
}

impl<'a, T> Render for &'a T
where
    T: RenderRef,
{
    fn render(self) -> VNode {
        RenderRef::render_ref(self)
    }
}

impl Render for String {
    fn render(self) -> VNode {
        text(self)
    }
}

impl<'a> Render for &'a String {
    fn render(self) -> VNode {
        text(self)
    }
}

impl<'a> Render for &'a str {
    fn render(self) -> VNode {
        text(self)
    }
}

impl<T> Render for Shared<T>
where
    T: RenderRef,
{
    fn render(self) -> VNode {
        self.0.as_ref().render_ref()
    }
}

impl Render for TagBuilder {
    fn render(self) -> VNode {
        self.build()
    }
}

impl Render for VText {
    fn render(self) -> VNode {
        VNode::Text(self)
    }
}

pub trait DomExtend: Sized {
    fn extend(self, parent: &mut TagBuilder);
}

impl<R: Render> DomExtend for R {
    fn extend(self, parent: &mut TagBuilder) {
        parent.add_child(self.render())
    }
}

impl DomExtend for VNode {
    fn extend(self, parent: &mut TagBuilder) {
        parent.add_child(self)
    }
}

impl<T: DomExtend> DomExtend for Option<T> {
    fn extend(self, parent: &mut TagBuilder) {
        if let Some(inner) = self {
            inner.extend(parent);
        }
    }
}

impl<T: DomExtend> DomExtend for Vec<T> {
    fn extend(self, parent: &mut TagBuilder) {
        for item in self {
            item.extend(parent);
        }
    }
}

impl<A, B> DomExtend for (A, B)
where
    A: DomExtend,
    B: DomExtend,
{
    fn extend(self, parent: &mut TagBuilder) {
        self.0.extend(parent);
        self.1.extend(parent);
    }
}

impl<A, B, C> DomExtend for (A, B, C)
where
    A: DomExtend,
    B: DomExtend,
    C: DomExtend,
{
    fn extend(self, parent: &mut TagBuilder) {
        self.0.extend(parent);
        self.1.extend(parent);
        self.2.extend(parent);
    }
}

impl<A, B, C, D> DomExtend for (A, B, C, D)
where
    A: DomExtend,
    B: DomExtend,
    C: DomExtend,
    D: DomExtend,
{
    fn extend(self, parent: &mut TagBuilder) {
        self.0.extend(parent);
        self.1.extend(parent);
        self.2.extend(parent);
        self.3.extend(parent);
    }
}

impl<A, B, C, D, E> DomExtend for (A, B, C, D, E)
where
    A: DomExtend,
    B: DomExtend,
    C: DomExtend,
    D: DomExtend,
    E: DomExtend,
{
    fn extend(self, parent: &mut TagBuilder) {
        self.0.extend(parent);
        self.1.extend(parent);
        self.2.extend(parent);
        self.3.extend(parent);
        self.4.extend(parent);
    }
}

#[inline]
pub fn text(text: impl Into<Str>) -> VNode {
    VNode::Text(VText::new(text.into()))
}

#[inline]
pub fn s(text: impl Into<Str>) -> VNode {
    VNode::Text(VText::new(text.into()))
}

#[inline]
pub fn text_static(text: &'static str) -> VNode {
    VNode::Text(VText::new(Str::stat(text)))
}

#[inline]
pub fn tag(tag: Tag) -> TagBuilder {
    TagBuilder::new(tag)
}

#[inline]
pub fn tag_with(tag: Tag, child: impl DomExtend) -> TagBuilder {
    self::tag(tag).and(child)
}

#[inline]
pub fn b() -> TagBuilder {
    TagBuilder::new(Tag::B)
}

#[inline]
pub fn div() -> TagBuilder {
    TagBuilder::new(Tag::Div)
}

#[inline]
pub fn div_with(child: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::Div).and(child)
}

#[inline]
pub fn i() -> TagBuilder {
    TagBuilder::new(Tag::I)
}

#[inline]
pub fn img(src: impl Into<Str>) -> TagBuilder {
    TagBuilder::new(Tag::Img).attr(Attr::Src, src)
}

#[inline]
pub fn span() -> TagBuilder {
    TagBuilder::new(Tag::Span)
}

#[inline]
pub fn span_with(child: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::Span).and(child)
}

#[inline]
pub fn p() -> TagBuilder {
    TagBuilder::new(Tag::P)
}

#[inline]
pub fn p_with(child: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::P).and(child)
}

#[inline]
pub fn header() -> TagBuilder {
    TagBuilder::new(Tag::Header)
}

#[inline]
pub fn button() -> TagBuilder {
    TagBuilder::new(Tag::Button)
}

#[inline]
pub fn button_with(child: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::Button).and(child)
}

#[inline]
pub fn hr() -> TagBuilder {
    TagBuilder::new(Tag::Hr)
}

#[inline]
pub fn h1() -> TagBuilder {
    TagBuilder::new(Tag::H1)
}

#[inline]
pub fn h2() -> TagBuilder {
    TagBuilder::new(Tag::H2)
}

#[inline]
pub fn h2_with(child: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::H2).and(child)
}

#[inline]
pub fn h3() -> TagBuilder {
    TagBuilder::new(Tag::H3)
}

#[inline]
pub fn h4() -> TagBuilder {
    TagBuilder::new(Tag::H4)
}

pub fn h5() -> TagBuilder {
    TagBuilder::new(Tag::H5)
}

#[inline]
pub fn ul() -> TagBuilder {
    TagBuilder::new(Tag::Ul)
}

#[inline]
pub fn li() -> TagBuilder {
    TagBuilder::new(Tag::Li)
}

#[inline]
pub fn li_with(child: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::Li).and(child)
}

#[inline]
pub fn small() -> TagBuilder {
    TagBuilder::new(Tag::Small)
}

#[inline]
pub fn small_with(child: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::Small).and(child)
}

#[inline]
pub fn strong() -> TagBuilder {
    TagBuilder::new(Tag::Strong)
}

#[inline]
pub fn strong_with(child: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::Strong).and(child)
}

#[inline]
pub fn table() -> TagBuilder {
    TagBuilder::new(Tag::Table)
}

#[inline]
pub fn tr() -> TagBuilder {
    TagBuilder::new(Tag::Tr)
}

#[inline]
pub fn tr_with(content: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::Tr).and(content)
}

pub fn th() -> TagBuilder {
    TagBuilder::new(Tag::Th)
}

#[inline]
pub fn th_with(content: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::Th).and(content)
}

#[inline]
pub fn td() -> TagBuilder {
    TagBuilder::new(Tag::Td)
}

#[inline]
pub fn td_with(content: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::Td).and(content)
}

#[inline]
pub fn a() -> TagBuilder {
    TagBuilder::new(Tag::A)
}

#[inline]
pub fn a_with(child: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::A).and(child)
}

// Form related.

#[inline]
pub fn form() -> TagBuilder {
    TagBuilder::new(Tag::Form)
}

#[inline]
pub fn label() -> TagBuilder {
    TagBuilder::new(Tag::Label)
}

#[inline]
pub fn label_with(child: impl DomExtend) -> TagBuilder {
    TagBuilder::new(Tag::Label).and(child)
}

#[inline]
pub fn input() -> TagBuilder {
    TagBuilder::new(Tag::Input)
}

#[inline]
pub fn textarea() -> TagBuilder {
    TagBuilder::new(Tag::TextArea)
}

#[inline]
pub fn select() -> TagBuilder {
    TagBuilder::new(Tag::Select)
}

#[inline]
pub fn option() -> TagBuilder {
    TagBuilder::new(Tag::Option)
}
