use crate::dom::AbortGuard;

/// The "global" context for an app.
pub(crate) struct AppContext {
    // /// The highest assigned event handler id.
    // max_event_handler_id: EventHandlerId,
    // /// All event types that have handlers.
    // /// Events not in this list are ignored.
    // active_events: FnvHashMap<EventType, u32>,
}

static mut ACTIVE_CONTEXT: Option<&mut AppContext> = None;

impl AppContext {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            // max_event_handler_id: EventHandlerId::new_zero(),
            // active_events: FnvHashMap::default(),
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
