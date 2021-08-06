use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{closure::Closure, JsCast};

pub fn now() -> f64 {
    js_sys::Date::now()
}

pub fn window() -> web_sys::Window {
    // TODO: use OnceCell
    web_sys::window().expect("Could not get window")
}

pub fn document() -> web_sys::Document {
    // TODO: use OnceCell
    window().document().expect("Could not get document")
}

/// Get the current window URL pathname.
/// Equivalent to Javascript: `window.location.pathname`;
pub fn url_path() -> String {
    window().location().pathname().unwrap()
}

pub fn input_event_target(ev: web_sys::Event) -> Option<web_sys::HtmlInputElement> {
    ev.current_target()?
        .dyn_into::<web_sys::HtmlInputElement>()
        .ok()
}

pub fn input_event_value(ev: web_sys::Event) -> Option<String> {
    input_event_target(ev).map(|x| x.value())
}

pub fn input_event_checkbox_value(ev: web_sys::Event) -> Option<bool> {
    ev.current_target()?
        .dyn_ref::<web_sys::HtmlInputElement>()
        .map(|x| x.checked())
}

pub fn textarea_input_value(ev: web_sys::Event) -> Option<String> {
    let v = ev
        .current_target()?
        .dyn_ref::<web_sys::HtmlTextAreaElement>()?
        .value();
    Some(v)
}

pub fn query_selector(sel: &str) -> Option<web_sys::Node> {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector(sel)
        .unwrap_or(None)
        .map(|x| x.unchecked_into())
}

/// A timeout future.
pub(crate) struct Timeout {
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
            let delay = (s.target - now()) as i32;
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

pub(crate) fn timeout(delay: std::time::Duration) -> Timeout {
    let target = now() + delay.as_millis() as f64;
    Timeout::new(target)
}

// pub struct KeyboardSubscription<M> {
//     target: web_sys::EventTarget,
//     closure: Closure<dyn Fn(web_sys::Event)>,
//     queue: Rc<RefCell<VecDeque<M>>>,
// }

// impl<M> futures::stream::Stream for KeyboardSubscription<M> {
//     type Item = M;

//     fn poll_next(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Option<Self::Item>> {
//         let s = self.get_mut();

//         if s.closure.is_none() {

//             std::task::Poll::Pending
//         } else {
//             if let Some(msg) = s.queue.borrow_mut().pop_front() {
//                 std::task::Poll::Ready(Some(msg))
//             } else {
//                 std::task::Poll::Pending
//             }
//         }
//     }
// }
