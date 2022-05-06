pub mod msg;

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{
    context::{AppContext, AppContextRef},
    dom::TagBuilder,
};

struct State<C> {
    state: Option<C>,
}

pub struct Context<'a, C> {
    state: &'a Rc<RefCell<State<C>>>,
}

impl<'a, C: Component> Context<'a, C> {
    pub fn handle(&self) -> Handle<C> {
        Handle {
            state: Rc::downgrade(self.state),
            context: AppContext::get_ref(),
        }
    }
}

pub struct Handle<C: Component> {
    state: Weak<RefCell<State<C>>>,
    context: AppContextRef,
}

impl<C: Component> Clone for Handle<C> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            context: self.context.clone(),
        }
    }
}

impl<C: Component> Handle<C> {
    // FIXME: proper error instead of ()
    pub fn apply(&self, f: impl FnOnce(&mut C, Context<'_, C>)) {
        if let Some(state) = self.state.upgrade() {
            let mut borrow = state.borrow_mut();
            if let Some(data) = borrow.state.as_mut() {
                self.context.with(|| {
                    f(data, Context { state: &state });
                });
            } else {
                #[cfg(debug_assertions)]
                tracing::warn!(
                    component=%std::any::type_name::<C>(),
                    "Tried to send message to uninitialized component"
                );
            }
        } else {
            #[cfg(debug_assertions)]
            tracing::warn!(
                component=%std::any::type_name::<C>(),
                "Tried to send message to dropped component"
            );
        }
    }
}

pub trait Component: Sized + 'static {
    type Properties;

    fn init(props: Self::Properties, ctx: Context<'_, Self>) -> Self;
    fn render(&mut self, ctx: Context<'_, Self>) -> TagBuilder;

    fn build(props: Self::Properties) -> crate::dom::View {
        build_component::<Self>(props)
    }
}

pub fn build_component<C: Component>(props: C::Properties) -> crate::dom::View {
    let comp = Rc::new(RefCell::new(State { state: None }));
    let mut state = C::init(props, Context { state: &comp });

    let mut node = {
        let mut borrow = comp.borrow_mut();
        let node = state.render(Context { state: &comp });
        borrow.state = Some(state);

        node
    };

    node.add_after_remove(move || {
        std::mem::drop(comp);
    });
    node.into()
}
