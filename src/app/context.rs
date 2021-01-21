use std::marker::PhantomData;

use futures::{FutureExt, StreamExt};
use wasm_bindgen_futures::spawn_local;

use crate::into_any_box;

use super::{
    component_manager::ComponentId,
    effect::{Callback, EffectFuture, EffectGuard},
    state::AppState,
};

pub struct Context<'a, M> {
    app: &'a mut AppState,
    component_id: ComponentId,

    skip_render: bool,
    effects: Vec<EffectGuard>,

    _marker: PhantomData<&'a M>,
}

impl<'a, M> Context<'a, M> {
    pub(crate) fn new(app: &'a mut AppState, component_id: ComponentId) -> Self {
        Self {
            app,
            component_id,
            skip_render: false,
            effects: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub(crate) fn is_skip_render(&self) -> bool {
        self.skip_render
    }

    pub(crate) fn take_effects(&mut self) -> Vec<EffectGuard> {
        std::mem::take(&mut self.effects)
    }

    pub fn skip_render(&mut self) {
        self.skip_render = true;
    }

    // pub fn navigate(&mut self, route: String) {
    //     match &mut self.state {
    //         ContextState::Direct { effect } => effect.and(Effect::Navigate(route)),
    //         ContextState::Nested { effect } => effect.and(Effect::Navigate(route)),
    //     }
    // }

    pub fn run_opt<F>(&mut self, f: F) -> EffectGuard
    where
        M: Unpin + 'static,
        F: std::future::Future<Output = Option<M>> + 'static,
    {
        let handle = self.app.make_component_handle(self.component_id);
        let mapped = f.map(|x| x.map(into_any_box));

        let (guarded, guard) = EffectFuture::new(handle, Box::pin(mapped));

        spawn_local(guarded);
        guard
    }

    pub fn run_opt_unguarded<F>(&mut self, f: F)
    where
        M: Unpin + 'static,
        F: std::future::Future<Output = Option<M>> + 'static,
    {
        let guard = self.run_opt(f);
        self.effects.push(guard);
    }

    pub fn run<F>(&mut self, f: F) -> EffectGuard
    where
        M: Unpin + 'static,
        F: std::future::Future<Output = M> + 'static,
    {
        self.run_opt(f.map(Some))
    }

    pub fn run_ungarded<F>(&mut self, f: F)
    where
        M: Unpin + 'static,
        F: std::future::Future<Output = M> + 'static,
    {
        self.run_opt_unguarded(f.map(Some));
    }

    pub fn run_map<T, F, FM>(&mut self, f: F, mapper: FM) -> EffectGuard
    where
        M: Unpin + 'static,
        F: std::future::Future<Output = T> + 'static,
        FM: Fn(T) -> M + 'static,
    {
        self.run(f.map(mapper))
    }

    pub fn run_map_ungarded<T, F, FM>(&mut self, f: F, mapper: FM)
    where
        M: Unpin + 'static,
        F: std::future::Future<Output = T> + 'static,
        FM: Fn(T) -> M + 'static,
    {
        self.run_opt_unguarded(f.map(move |x| Some(mapper(x))));
    }

    pub fn subscribe<S>(&mut self, mut stream: S) -> EffectGuard
    where
        M: 'static,
        S: futures::stream::Stream<Item = M> + Unpin + 'static,
    {
        let app = self.app.make_component_handle(self.component_id);
        let (guard, handle) = EffectGuard::new();
        spawn_local(async move {
            while let Some(msg) = stream.next().await {
                if !handle.is_cancelled() {
                    app.send_message(Box::new(msg));
                }
            }
        });
        guard
    }

    pub fn subscribe_unguarded<S>(&mut self, stream: S)
    where
        M: 'static,
        S: futures::stream::Stream<Item = M> + Unpin + 'static,
    {
        let guard = self.subscribe(stream);
        self.effects.push(guard);
    }

    pub fn callback(&mut self) -> Callback<M>
    where
        M: 'static,
    {
        Callback::new(self.app.make_component_handle(self.component_id))
    }

    pub fn callback_map<T, F>(&mut self, mapper: F) -> Callback<T>
    where
        M: 'static,
        T: 'static,
        F: Fn(T) -> M + 'static,
    {
        self.callback().map(mapper)
    }

    pub fn timeout(&mut self, msg: M, delay: std::time::Duration) -> EffectGuard
    where
        M: Unpin + 'static,
    {
        self.run(async move {
            Box::pin(crate::util::timeout(delay)).await;
            msg
        })
    }
}
