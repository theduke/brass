use crate::{
    dom::TagBuilder,
    effect::{spawn_guarded, EffectGuard},
};

use super::{Component, Context};

pub trait MsgComponent: Sized + 'static {
    type Properties;
    type Msg;

    fn init(props: Self::Properties, ctx: Context<Self>) -> Self;
    fn update(&mut self, msg: Self::Msg, ctx: Context<Self>);
    fn render(&mut self, ctx: Context<Self>) -> TagBuilder;
}

impl<C: MsgComponent> Component for C {
    type Properties = <Self as MsgComponent>::Properties;

    fn init(props: Self::Properties, ctx: Context<'_, Self>) -> Self {
        MsgComponent::init(props, ctx)
    }

    fn render(&mut self, ctx: Context<'_, Self>) -> TagBuilder {
        MsgComponent::render(self, ctx)
    }
}

impl<C: MsgComponent> Context<'_, C> {
    pub fn callback(&self) -> impl Fn(C::Msg) {
        let h = self.handle();
        move |msg| h.send(msg)
    }

    pub fn callback_msg(&self, f: impl Fn() -> C::Msg + 'static) -> impl Fn()
    where
        C::Msg: 'static,
    {
        let h = self.handle();
        move || {
            h.send(f());
        }
    }

    pub fn callback_msg_clone(&self, msg: C::Msg) -> impl Fn()
    where
        C::Msg: Clone,
    {
        let h = self.handle();
        move || {
            h.send(msg.clone());
        }
    }

    pub fn on<E, F: Fn(E) -> C::Msg>(&self, f: F) -> impl Fn(E) {
        let handle = self.handle();
        move |e: E| {
            let msg = f(e);
            handle.send(msg);
        }
    }

    pub fn on_opt<E, F: Fn(E) -> Option<C::Msg>>(&self, f: F) -> impl Fn(E) {
        let handle = self.handle();
        move |e: E| {
            if let Some(msg) = f(e) {
                handle.send(msg);
            }
        }
    }

    pub fn spawn(&self, f: impl std::future::Future<Output = C::Msg> + 'static) -> EffectGuard {
        let handle = self.handle();
        spawn_guarded(async move {
            let msg = f.await;
            handle.send(msg);
        })
    }

    pub fn spawn_map<O, F, M>(&self, f: F, mapper: M) -> EffectGuard
    where
        F: std::future::Future<Output = O> + 'static,
        M: Fn(O) -> C::Msg + 'static,
    {
        let handle = self.handle();
        spawn_guarded(async move {
            let out = f.await;
            let msg = mapper(out);
            handle.send(msg);
        })
    }
}

impl<C: MsgComponent> super::Handle<C> {
    pub fn send(&self, msg: C::Msg) {
        self.apply(move |state, ctx| {
            state.update(msg, ctx);
        })
    }

    pub fn callback(&self, f: impl Fn() -> C::Msg) -> impl Fn() {
        let s = self.clone();
        move || s.send(f())
    }

    pub fn callback_msg(&self, msg: C::Msg) -> impl Fn()
    where
        C::Msg: Clone,
    {
        let s = self.clone();
        move || s.send(msg.clone())
    }

    pub fn on<E, F: Fn(E) -> C::Msg>(&self, f: F) -> impl Fn(E) {
        let handle = self.clone();
        move |e: E| {
            let msg = f(e);
            handle.send(msg);
        }
    }

    pub fn on_opt<E, F: Fn(E) -> Option<C::Msg>>(&self, f: F) -> impl Fn(E) {
        let handle = self.clone();
        move |e: E| {
            if let Some(msg) = f(e) {
                handle.send(msg);
            }
        }
    }
}
