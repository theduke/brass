use wasm_bindgen::{prelude::Closure, JsCast};

use crate::dom::{AbortGuard, Ev};

/// The "global" context for an app.
pub(crate) struct AppContext {
    /// All event types that have handlers.
    /// Events not in this list are ignored.
    active_events: Vec<EventHandler>,
    event_freelist: Vec<EventHandlerId>,
}

static mut ACTIVE_CONTEXT: Option<&mut AppContext> = None;

impl AppContext {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            active_events: Vec::new(),
            event_freelist: Vec::new(),
        })
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
        if let Some(handler) = self
            .active_events
            .get_mut(id.as_usize())
            .and_then(|x| x.handler.as_mut())
        {
            (handler)(event);
        } else {
            tracing::error!("invoked event handler with invalid id");
        }
    }

    pub fn create_event_listener<H>(
        event: Ev,
        callback: H,
        target: &web_sys::Node,
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
            h
        } else {
            let id = EventHandlerId(inner.active_events.len());

            let id2 = id.clone();
            let inner2 = Self::get_mut();
            let boxed =
                Box::new(move |event: web_sys::Event| inner2.invoke_event_handler(id2, event))
                    as Box<dyn FnMut(web_sys::Event)>;
            let closure = wasm_bindgen::closure::Closure::wrap(boxed);

            let h = EventHandler {
                id,
                closure,
                handler: Some(callback),
            };

            inner.active_events.push(h);

            inner.active_events.last_mut().unwrap()
        };

        crate::web::add_event_lister(target, event, handler.closure.as_ref().unchecked_ref());

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
        } else {
            let handler = inner
                .active_events
                .get_mut(index)
                .expect("Invalid event handler id provided");
            // Drop the callback to free up references.
            handler.handler.take();
            inner.event_freelist.push(id);
        }
    }

    pub fn spawn_abortable<F>(f: F) -> AbortGuard
    where
        F: std::future::Future<Output = ()> + 'static,
    {
        let (handle, reg) = futures::future::AbortHandle::new_pair();
        let f = futures::future::Abortable::new(f, reg);

        Self::spawn_unguarded(async move {
            f.await.ok();
        });

        AbortGuard(handle)
    }

    pub fn spawn_unguarded<F>(f: F)
    where
        F: std::future::Future<Output = ()> + 'static,
    {
        let context = Self::get_mut();

        // TODO: add a spawn_local_boxed method to wasm_bindgen_futures to
        // allow for a non-generic helper function that doesn't do double-boxing
        // (spawn_local boxes the future)
        wasm_bindgen_futures::spawn_local(async move {
            ContextFuture::new(context, f).await;
        });
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
