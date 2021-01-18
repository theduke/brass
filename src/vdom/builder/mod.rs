use crate::{
    dom::{Attr, Event, Tag},
    Component,
};

use super::{
    event_manager::EventCallbackId, EventHandler, Listener, VComponent, VNode, VTag, VText,
};

pub fn component<C: Component, M>(props: C::Properties) -> VNode<M> {
    VNode::Component(VComponent::new::<C>(props))
}

pub struct TagBuilder<M> {
    tag: VTag<M>,
}

impl<M> TagBuilder<M> {
    pub fn new(tag: Tag) -> Self {
        Self {
            tag: VTag::new(tag),
        }
    }

    pub fn wrap<P: Into<TagBuilder<M>>>(self, parent: P) -> TagBuilder<M> {
        parent.into().and(self)
    }

    #[inline]
    pub fn class(self, cls: &str) -> Self {
        self.attr(Attr::Class, cls)
    }

    pub fn attr(mut self, attribute: Attr, value: impl Into<String>) -> Self {
        self.tag.attributes.insert(attribute.into(), value.into());
        self
    }

    pub fn attr_if(mut self, flag: bool, attribute: Attr, value: impl Into<String>) -> Self {
        if flag {
            self.tag.attributes.insert(attribute.into(), value.into());
        }
        self
    }

    #[inline]
    pub fn and(mut self, child: impl DomExtend<M>) -> Self {
        child.extend(&mut self);
        self
    }

    pub fn and_iter<T, I>(mut self, iter: I) -> Self
    where
        T: DomExtend<M>,
        I: IntoIterator<Item = T>,
    {
        for item in iter.into_iter() {
            item.extend(&mut self);
        }

        self
    }

    pub fn and_if<F, T>(mut self, flag: bool, f: F) -> Self
    where
        T: DomExtend<M>,
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
        T: DomExtend<M>,
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

    pub fn add_child(&mut self, child: impl Into<VNode<M>>) {
        self.tag.children.push(child.into());
    }

    pub fn build(self) -> VNode<M> {
        VNode::Tag(self.tag)
    }

    pub fn on(mut self, event: Event, handler: fn(web_sys::Event) -> Option<M>) -> Self {
        self.tag.listeners.push(Listener {
            event,
            handler: EventHandler::Fn(handler),
            callback_id: EventCallbackId::new_null(),
        });

        self
    }

    pub fn on_captured(
        mut self,
        event: Event,
        handler: impl Fn(web_sys::Event) -> Option<M> + 'static,
    ) -> Self {
        self.tag.listeners.push(Listener {
            event,
            handler: EventHandler::Closure(std::rc::Rc::new(handler)),
            callback_id: EventCallbackId::new_null(),
        });

        self
    }
}

impl<M> From<TagBuilder<M>> for VNode<M> {
    fn from(b: TagBuilder<M>) -> Self {
        b.build()
    }
}

pub trait DomExtend<M>: Sized {
    fn extend(self, parent: &mut TagBuilder<M>);
}

impl<M> DomExtend<M> for String {
    fn extend(self, parent: &mut TagBuilder<M>) {
        parent.add_child(text(self))
    }
}

impl<'a, M> DomExtend<M> for &'a String {
    fn extend(self, parent: &mut TagBuilder<M>) {
        parent.add_child(text(self))
    }
}

impl<'a, M> DomExtend<M> for &'a str {
    fn extend(self, parent: &mut TagBuilder<M>) {
        parent.add_child(text(self))
    }
}

impl<M> DomExtend<M> for TagBuilder<M> {
    fn extend(self, parent: &mut TagBuilder<M>) {
        parent.add_child(self.build())
    }
}

impl<M> DomExtend<M> for VText {
    fn extend(self, parent: &mut TagBuilder<M>) {
        parent.add_child(VNode::Text(self))
    }
}

impl<M> DomExtend<M> for VNode<M> {
    fn extend(self, parent: &mut TagBuilder<M>) {
        parent.add_child(self)
    }
}

impl<M, T: DomExtend<M>> DomExtend<M> for Option<T> {
    fn extend(self, parent: &mut TagBuilder<M>) {
        if let Some(inner) = self {
            inner.extend(parent);
        }
    }
}

impl<M, T: DomExtend<M>> DomExtend<M> for Vec<T> {
    fn extend(self, parent: &mut TagBuilder<M>) {
        for item in self {
            item.extend(parent);
        }
    }
}

impl<M, A, B> DomExtend<M> for (A, B)
where
    A: DomExtend<M>,
    B: DomExtend<M>,
{
    fn extend(self, parent: &mut TagBuilder<M>) {
        self.0.extend(parent);
        self.1.extend(parent);
    }
}

impl<M, A, B, C> DomExtend<M> for (A, B, C)
where
    A: DomExtend<M>,
    B: DomExtend<M>,
    C: DomExtend<M>,
{
    fn extend(self, parent: &mut TagBuilder<M>) {
        self.0.extend(parent);
        self.1.extend(parent);
        self.2.extend(parent);
    }
}

impl<M, A, B, C, D> DomExtend<M> for (A, B, C, D)
where
    A: DomExtend<M>,
    B: DomExtend<M>,
    C: DomExtend<M>,
    D: DomExtend<M>,
{
    fn extend(self, parent: &mut TagBuilder<M>) {
        self.0.extend(parent);
        self.1.extend(parent);
        self.2.extend(parent);
        self.3.extend(parent);
    }
}

impl<M, A, B, C, D, E> DomExtend<M> for (A, B, C, D, E)
where
    A: DomExtend<M>,
    B: DomExtend<M>,
    C: DomExtend<M>,
    D: DomExtend<M>,
    E: DomExtend<M>,
{
    fn extend(self, parent: &mut TagBuilder<M>) {
        self.0.extend(parent);
        self.1.extend(parent);
        self.2.extend(parent);
        self.3.extend(parent);
        self.4.extend(parent);
    }
}

#[inline]
pub fn text<M>(text: impl Into<VText>) -> VNode<M> {
    VNode::Text(text.into())
}

#[inline]
pub fn tag<M>(tag: Tag) -> TagBuilder<M> {
    TagBuilder::new(tag)
}

#[inline]
pub fn tag_with<M>(tag: Tag, child: impl DomExtend<M>) -> TagBuilder<M> {
    self::tag(tag).and(child)
}

#[inline]
pub fn div<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Div)
}

#[inline]
pub fn div_with<M>(child: impl DomExtend<M>) -> TagBuilder<M> {
    TagBuilder::new(Tag::Div).and(child)
}

#[inline]
pub fn img<M>(src: &str) -> TagBuilder<M> {
    TagBuilder::new(Tag::Img).attr(Attr::Src, src)
}

#[inline]
pub fn span<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Span)
}

#[inline]
pub fn span_with<M>(child: impl DomExtend<M>) -> TagBuilder<M> {
    TagBuilder::new(Tag::Span).and(child)
}

#[inline]
pub fn p<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::P)
}

#[inline]
pub fn p_with<M>(child: impl DomExtend<M>) -> TagBuilder<M> {
    TagBuilder::new(Tag::P).and(child)
}

#[inline]
pub fn button<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Button)
}

#[inline]
pub fn button_with<M>(child: impl DomExtend<M>) -> TagBuilder<M> {
    TagBuilder::new(Tag::Button).and(child)
}

#[inline]
pub fn h1<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::H1)
}

#[inline]
pub fn h2<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::H2)
}

#[inline]
pub fn h2_with<M>(child: impl DomExtend<M>) -> TagBuilder<M> {
    TagBuilder::new(Tag::H2).and(child)
}

#[inline]
pub fn h3<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::H3)
}

#[inline]
pub fn h4<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::H4)
}

#[inline]
pub fn ul<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Ul)
}

#[inline]
pub fn li<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Li)
}

#[inline]
pub fn li_with<M>(child: impl DomExtend<M>) -> TagBuilder<M> {
    TagBuilder::new(Tag::Li).and(child)
}

#[inline]
pub fn small<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Small)
}

#[inline]
pub fn small_with<M>(child: impl DomExtend<M>) -> TagBuilder<M> {
    TagBuilder::new(Tag::Small).and(child)
}

#[inline]
pub fn strong<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Strong)
}

#[inline]
pub fn strong_with<M>(child: impl DomExtend<M>) -> TagBuilder<M> {
    TagBuilder::new(Tag::Strong).and(child)
}

#[inline]
pub fn table<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Table)
}

#[inline]
pub fn tr<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Tr)
}

pub fn th<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Th)
}

#[inline]
pub fn td<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Td)
}

#[inline]
pub fn a<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::A)
}

#[inline]
pub fn a_with<M>(child: impl DomExtend<M>) -> TagBuilder<M> {
    TagBuilder::new(Tag::A).and(child)
}

// Form related.

#[inline]
pub fn label<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Label)
}

#[inline]
pub fn label_with<M>(child: impl DomExtend<M>) -> TagBuilder<M> {
    TagBuilder::new(Tag::Label).and(child)
}

#[inline]
pub fn input<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::Input)
}

#[inline]
pub fn textarea<M>() -> TagBuilder<M> {
    TagBuilder::new(Tag::TextArea)
}
