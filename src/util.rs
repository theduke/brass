use std::{cell::RefCell, future::Ready, rc::Rc};

use futures::future::Pending;
use wasm_bindgen::{closure::Closure, JsCast};

use crate::now;

pub fn window() -> web_sys::Window {
    // TODO: use OnceCell
    web_sys::window().expect("Could not get window")
}

/// A "HashMap" that internally uses a Vec for storage.
/// This is a good storage data structure for attribute lists, since it has
/// stable ordering and most dom elements have very few attributes.

#[derive(Clone, Debug)]
pub struct VecMap<T> {
    items: Vec<T>,
}

impl<T> Default for VecMap<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T> VecMap<T>
where
    T: Eq + PartialEq,
{
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }
}

/// A timeout future.
pub struct Timeout {
    closure: Option<Closure<dyn Fn()>>,
    // Javascript target time.
    target: f64,
    is_complete: Rc<RefCell<bool>>,
}

impl Timeout {
    fn new(target: f64) -> Self {
        Self {
            closure: None,
            target,
            is_complete: Rc::new(RefCell::new(false)),
        }
    }
}

impl std::future::Future for Timeout {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let s = self.get_mut();

        if s.closure.is_none() {
            let delay = (s.target - super::now()) as i32;
            if delay < 0 {
                // Already done.
                *s.is_complete.borrow_mut() = true;
                return std::task::Poll::Ready(());
            }

            // Schedule.
            let waker = cx.waker().to_owned();
            let flag = s.is_complete.clone();
            let c = Closure::wrap(Box::new(move || {
                *flag.borrow_mut() = true;
                waker.clone().wake();
            }) as Box<dyn Fn()>);
            window()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    c.as_ref().unchecked_ref(),
                    delay,
                )
                .expect("Could not set timeout");
            s.closure = Some(c);

            std::task::Poll::Pending
        } else {
            if *s.is_complete.borrow_mut() {
                std::task::Poll::Ready(())
            } else {
                std::task::Poll::Pending
            }
        }
    }
}

pub fn timeout(delay: std::time::Duration) -> Timeout {
    let target = now() + delay.as_millis() as f64;
    Timeout::new(target)
}
