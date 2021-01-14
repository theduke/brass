pub mod component;
mod event_manager;
mod patch;

mod builder;
pub use builder::*;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::dom;

// TODO: use cow or https://github.com/maciejhirsz/beef ?
#[derive(Debug)]
pub struct VText {
    value: String,
    node: Option<web_sys::Node>,
}

impl VText {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            node: None,
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

pub type StaticEventHandler<M> = fn(web_sys::Event) -> Option<M>;
// TODO: use box instead?
pub type ClosureEventHandler<M> = Rc<dyn Fn(web_sys::Event) -> Option<M>>;

pub enum EventHandler<M> {
    Fn(StaticEventHandler<M>),
    Closure(ClosureEventHandler<M>),
}

impl<M> EventHandler<M> {
    pub fn map<M2, F>(self, mapper: F) -> EventHandler<M2>
    where
        M: 'static,
        M2: 'static,
        F: Fn(M) -> M2 + Clone + 'static,
    {
        match self {
            EventHandler::Fn(f) => {
                EventHandler::Closure(Rc::new(move |event| f(event).map(mapper.clone())))
            }
            EventHandler::Closure(f) => {
                EventHandler::Closure(Rc::new(move |event| f(event).map(mapper.clone())))
            }
        }
    }
}

impl<M> Clone for EventHandler<M> {
    fn clone(&self) -> Self {
        match self {
            Self::Fn(f) => Self::Fn(*f),
            Self::Closure(f) => Self::Closure(f.clone()),
        }
    }
}

impl<M> EventHandler<M> {
    pub fn invoke(&self, event: web_sys::Event) -> Option<M> {
        match self {
            EventHandler::Fn(f) => f(event),
            EventHandler::Closure(f) => (f)(event),
        }
    }
}

impl<M> PartialEq for EventHandler<M> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EventHandler::Fn(f1), EventHandler::Fn(f2)) => f1 == f2,
            _ => false,
        }
    }
}

impl<M> From<StaticEventHandler<M>> for EventHandler<M> {
    fn from(f: StaticEventHandler<M>) -> Self {
        Self::Fn(f)
    }
}

struct Listener<M> {
    event: dom::Event,
    handler: EventHandler<M>,
    callback_id: event_manager::EventCallbackId,
}

impl<M> Listener<M> {
    fn map<M2, F>(self, mapper: F) -> Listener<M2>
    where
        M: 'static,
        M2: 'static,
        F: Fn(M) -> M2 + Clone + 'static,
    {
        Listener {
            event: self.event,
            handler: self.handler.map(mapper),
            callback_id: self.callback_id,
        }
    }
}

impl<M> PartialEq for Listener<M> {
    fn eq(&self, other: &Self) -> bool {
        self.event == other.event && self.handler == other.handler
    }
}

pub struct VTag<M> {
    tag: dom::Tag,
    // TODO: use a faster hash map and a better key.
    attributes: HashMap<dom::Attr, String>,
    children: Vec<VNode<M>>,

    // TODO: should this be a u32 or a Rc<String> ?
    // TODO: implement keying support for the rendering
    key: Option<String>,
    listeners: Vec<Listener<M>>,
    node: Option<web_sys::Node>,
}

impl<M> VTag<M> {
    pub fn new(tag: dom::Tag) -> Self {
        Self {
            tag,
            attributes: HashMap::new(),
            children: Vec::new(),
            key: None,
            listeners: Vec::new(),
            node: None,
        }
    }
}

impl<M> std::fmt::Debug for VTag<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tag")
            .field("kind", &self.tag)
            .field("attributes", &self.attributes)
            .field("key", &self.key)
            // .field("listeners", &self.listeners)
            .field("node", &self.node)
            .finish()
    }
}

#[derive(Debug)]
pub enum VNode<M> {
    Empty,
    Text(VText),
    Tag(VTag<M>),
}

impl<M> VNode<M> {
    pub fn map<M2, F>(self, mapper: F) -> VNode<M2>
    where
        M: 'static,
        M2: 'static,
        F: Fn(M) -> M2 + Clone + 'static,
    {
        match self {
            VNode::Empty => VNode::Empty,
            VNode::Text(txt) => VNode::Text(txt),
            VNode::Tag(tag) => VNode::Tag(VTag {
                tag: tag.tag,
                attributes: tag.attributes,
                children: tag
                    .children
                    .into_iter()
                    .map(|c| c.map(mapper.clone()))
                    .collect(),
                key: tag.key,
                listeners: tag
                    .listeners
                    .into_iter()
                    .map(|l| l.map(mapper.clone()))
                    .collect(),
                node: tag.node,
            }),
        }
    }
}

#[must_use]
pub struct EffectGuard {
    is_cancelled: Rc<RefCell<bool>>,
}

impl std::fmt::Debug for EffectGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectGuard").finish()
    }
}

pub struct EffectGuardHandle {
    is_cancelled: Rc<RefCell<bool>>,
}

impl EffectGuardHandle {
    pub fn is_cancelled(&self) -> bool {
        *self.is_cancelled.borrow()
    }
}

impl EffectGuard {
    fn new() -> (Self, EffectGuardHandle) {
        let flag = Rc::new(RefCell::new(false));
        (
            Self {
                is_cancelled: flag.clone(),
            },
            EffectGuardHandle {
                is_cancelled: flag.clone(),
            },
        )
    }

    pub fn is_cancelled(&self) -> bool {
        *self.is_cancelled.borrow()
    }
}

impl Drop for EffectGuard {
    fn drop(&mut self) {
        *self.is_cancelled.borrow_mut() = true;
    }
}

pub(crate) enum Effect<M> {
    None,
    SkipRender,
    // Delay {
    //     msg: M,
    //     delay_until: u64,
    //     guard: Option<EffectGuard>,
    // },
    Future {
        future: futures_core::future::LocalBoxFuture<'static, Option<M>>,
        guard: Option<EffectGuardHandle>,
    },
    Multi(Vec<Self>),
}

impl<M> Default for Effect<M> {
    fn default() -> Self {
        Self::None
    }
}

impl<M> Effect<M> {
    pub fn and(&mut self, eff: Self) {
        if let Effect::None = self {
            *self = eff
        } else if let Self::Multi(items) = self {
            items.push(eff)
        } else {
            let mut old = Effect::None;
            std::mem::swap(&mut old, self);
            *self = Self::Multi(vec![old, eff])
        }
    }

    pub fn map<M2, F>(self, mapper: F) -> Effect<M2>
    where
        M: 'static,
        F: Fn(M) -> M2 + Clone + 'static,
        M2: 'static,
    {
        match self {
            Effect::None => Effect::None,
            Effect::SkipRender => Effect::SkipRender,
            Effect::Future { future, guard } => Effect::Future {
                future: Box::pin(async move {
                    let out = future.await;
                    out.map(mapper)
                }),
                guard,
            },
            Effect::Multi(items) => {
                Effect::Multi(items.into_iter().map(|m| m.map(mapper.clone())).collect())
            }
        }
    }

    pub fn none() -> Self {
        Self::None
    }

    // pub fn delay_for(duration: std::time::Duration, msg: M) -> Self {
    //     Self::Delay{
    //         msg,
    //         delay_until: crate::now() as u64 + duration.as_secs(),
    //     }
    // }

    pub fn future<F>(f: F) -> (Self, EffectGuard)
    where
        F: std::future::Future<Output = Option<M>> + 'static,
    {
        let (guard, handle) = EffectGuard::new();
        (
            Self::Future {
                future: Box::pin(f),
                guard: Some(handle),
            },
            guard,
        )
    }

    pub fn future_unguarded<F>(f: F) -> Self
    where
        F: std::future::Future<Output = Option<M>> + 'static,
    {
        Self::Future {
            future: Box::pin(f),
            guard: None,
        }
    }
}
