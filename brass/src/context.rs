use futures::{future::LocalFutureObj, task::LocalSpawn};
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::{
    dom::{AbortGuard, Ev},
    effect::{IntervalGuard, TimeoutGuard},
};

/// The "global" context for an app.
pub(crate) struct AppContext {
    /// All event types that have handlers.
    /// Events not in this list are ignored.
    active_events: Vec<EventHandler>,
    event_freelist: Vec<EventHandlerId>,
    executor: futures::executor::LocalPool,
    spawner: futures::executor::LocalSpawner,
}

static mut ACTIVE_CONTEXT: Option<&mut AppContext> = None;

impl AppContext {
    pub fn new() -> Box<Self> {
        let executor = futures::executor::LocalPool::new();
        Box::new(Self {
            active_events: Vec::new(),
            event_freelist: Vec::new(),
            spawner: executor.spawner(),
            executor,
        })
    }

    pub(crate) fn leak_ref(mut self: Box<Self>) -> AppContextRef {
        let ptr = unsafe { &mut *(self.as_mut() as *mut Self) };
        let r = AppContextRef(ptr);
        std::mem::forget(self);
        r
    }

    pub fn with<O, F>(self: &mut Box<Self>, f: F) -> O
    where
        F: FnOnce() -> O,
    {
        self.enter();
        let out = f();
        Self::leave();

        out
    }

    // fn with_ref<O, F>(&mut self, f: F) -> O
    // where
    //     F: FnOnce() -> O,
    // {
    //     Self::enter(self);
    //     let out = f();
    //     Self::leave();
    //     out
    // }

    fn enter(&mut self) {
        unsafe {
            ACTIVE_CONTEXT = Some(&mut *(self as *mut Self));
        }
    }

    fn leave() {
        unsafe {
            ACTIVE_CONTEXT = None;
        }
    }

    fn get_mut() -> &'static mut Self {
        unsafe { ACTIVE_CONTEXT.as_mut().unwrap() }
    }

    fn invoke_event_handler(&mut self, id: EventHandlerId, event: web_sys::Event) {
        self.enter();
        if let Some(handler) = self
            .active_events
            .get_mut(id.as_usize())
            .and_then(|x| x.handler.as_mut())
        {
            (handler)(event);
        } else {
            tracing::error!("invoked event handler with invalid id");
            return;
        }
        self.process_futures();
        AppContext::leave();
    }

    pub fn create_event_listener<H>(
        event: Ev,
        callback: H,
        target: web_sys::EventTarget,
    ) -> EventHandlerRef
    where
        H: FnMut(web_sys::Event) + 'static,
    {
        let inner = Self::get_mut();

        let callback: Box<dyn FnMut(web_sys::Event)> = Box::new(callback);

        let handler = if let Some(h) = inner
            .event_freelist
            .pop()
            .and_then(|x| inner.active_events.get_mut(x.0))
        {
            h.handler = Some(callback);
            h.target = target;
            h.ty = event;
            h
        } else {
            let id = EventHandlerId(inner.active_events.len());

            let id2 = id.clone();
            let inner2 = Self::get_mut();
            let boxed = Box::new(move |event: web_sys::Event| {
                inner2.invoke_event_handler(id2, event);
            }) as Box<dyn FnMut(web_sys::Event)>;
            let closure = wasm_bindgen::closure::Closure::wrap(boxed);

            let h = EventHandler {
                id,
                closure,
                handler: Some(callback),
                target,
                ty: event,
            };

            inner.active_events.push(h);

            inner.active_events.last_mut().unwrap()
        };

        crate::web::add_event_lister(
            &handler.target,
            handler.ty,
            handler.closure.as_ref().unchecked_ref(),
        );

        EventHandlerRef(handler.id.clone())
    }

    fn return_event_handler(id: EventHandlerId) {
        let inner = Self::get_mut();
        let index = id.as_usize();

        // If a lot of event handlers are created, and the newly returned one is
        // the last id, drop it instead of enabling reuse.
        if inner.event_freelist.len() > 500 && index == inner.event_freelist.len() - 1 {
            let ev = inner.active_events.pop().unwrap();
            debug_assert_eq!(ev.id.as_usize(), index);

            ev.target
                .remove_event_listener_with_callback(
                    ev.ty.as_str(),
                    ev.closure.as_ref().unchecked_ref(),
                )
                .ok();
        } else {
            let handler = inner
                .active_events
                .get_mut(index)
                .expect("Invalid event handler id provided");
            // Drop the callback to free up references.
            handler.handler.take();
            inner.event_freelist.push(id);

            handler
                .target
                .remove_event_listener_with_callback(
                    handler.ty.as_str(),
                    handler.closure.as_ref().unchecked_ref(),
                )
                .ok();
        }
    }

    pub fn create_timeout(
        duration: std::time::Duration,
        f: impl FnOnce() + 'static,
    ) -> TimeoutGuard {
        let inner = Self::get_mut();
        // TODO: use callback cache
        let closure = Closure::once(move || {
            inner.enter();
            f();
            inner.process_futures();
            AppContext::leave();
        });

        let id = crate::web::window()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                duration.as_millis().try_into().unwrap(),
            )
            .unwrap();

        TimeoutGuard {
            _closure: closure,
            id,
        }
    }

    pub fn create_interval(
        duration: std::time::Duration,
        mut f: impl FnMut() + 'static,
    ) -> IntervalGuard {
        let inner = Self::get_mut();

        // TODO: use callback cache
        let closure = Closure::wrap(Box::new(move || {
            inner.enter();
            f();
            inner.process_futures();
            AppContext::leave();
        }) as Box<dyn FnMut()>);

        let id = crate::web::window()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                duration.as_millis().try_into().unwrap(),
            )
            .unwrap();

        IntervalGuard {
            _closure: closure,
            id,
        }
    }

    pub fn spawn_custom_executor_abortable<F>(f: F) -> AbortGuard
    where
        F: std::future::Future<Output = ()> + 'static,
    {
        let (handle, reg) = futures::future::AbortHandle::new_pair();
        let f = futures::future::Abortable::new(f, reg);

        Self::spawn_custom_executor_unguarded(async move {
            f.await.ok();
        });

        AbortGuard(handle)
    }

    pub fn spawn_custom_executor_unguarded<F>(f: F)
    where
        F: std::future::Future<Output = ()> + 'static,
    {
        let context = Self::get_mut();

        let obj = LocalFutureObj::new(Box::new(ContextFuture::new(Self::get_mut(), f)));
        context.spawner.spawn_local_obj(obj).unwrap();

        // // TODO: add a spawn_local_boxed method to wasm_bindgen_futures to
        // // allow for a non-generic helper function that doesn't do double-boxing
        // // (spawn_local boxes the future)
        // wasm_bindgen_futures::spawn_local(async move {
        //     ContextFuture::new(context, f).await;
        // });
    }

    pub fn spawn_external_abortable<F>(f: F) -> AbortGuard
    where
        F: std::future::Future<Output = ()> + 'static,
    {
        let (handle, reg) = futures::future::AbortHandle::new_pair();
        let f = futures::future::Abortable::new(f, reg);

        Self::spawn_external_unguarded(async move {
            f.await.ok();
        });

        AbortGuard(handle)
    }

    pub fn spawn_external_unguarded<F>(f: F)
    where
        F: std::future::Future<Output = ()> + 'static,
    {
        let context = Self::get_mut();
        wasm_bindgen_futures::spawn_local(async move {
            f.await;
            context.process_futures();
        });
    }

    pub(crate) fn process_futures(&mut self) {
        self.executor.run_until_stalled();
    }
}

pub struct AppContextRef(&'static mut AppContext);

impl AppContextRef {
    pub fn with<O, F: FnOnce() -> O>(&mut self, f: F) -> O {
        self.0.enter();
        let out = f();

        self.0.process_futures();
        AppContext::leave();
        out
    }

    pub async fn with_async<O, F: std::future::Future<Output = O>>(&mut self, f: F) -> O {
        self.0.enter();
        let out = f.await;

        self.0.process_futures();
        AppContext::leave();
        out
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct EventHandlerId(usize);

impl EventHandlerId {
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

struct EventHandler {
    id: EventHandlerId,
    closure: Closure<dyn FnMut(web_sys::Event)>,
    handler: Option<Box<dyn FnMut(web_sys::Event)>>,
    /// If Some(_), the event handler should be removed from the target element
    target: web_sys::EventTarget,
    ty: Ev,
}

pub struct EventHandlerRef(EventHandlerId);

impl Drop for EventHandlerRef {
    fn drop(&mut self) {
        AppContext::return_event_handler(self.0);
    }
}

pin_project_lite::pin_project! {
    struct ContextFuture<F> {
        context: &'static mut AppContext,
        #[pin]
        inner: F,
    }
}

impl<F> ContextFuture<F> {
    fn new(context: &'static mut AppContext, f: F) -> Self
    where
        F: std::future::Future,
    {
        Self { context, inner: f }
    }
}

impl<F> std::future::Future for ContextFuture<F>
where
    F: std::future::Future,
{
    type Output = F::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        this.context.enter();
        let poll = this.inner.poll(cx);
        AppContext::leave();
        poll
    }
}
