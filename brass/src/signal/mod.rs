// use std::{
//     cell::RefCell,
//     rc::{Rc, Weak},
//     sync::atomic::AtomicBool,
//     task::{Poll, Waker},
// };

// pub struct Mutable<V>(Rc<RefCell<MutableState<V>>>);

// struct MutableState<V> {
//     value: V,
//     wakers: Vec<MutableWaker>,
// }

// impl<V> MutableState<V> {
//     fn notify(&mut self) {
//         self.wakers.retain(|w| {
//             if !w.cancel.is_canceled() {
//                 w.waker.wake_by_ref();
//                 true
//             } else {
//                 false
//             }
//         });
//     }

//     fn set(&mut self, value: V) {
//         self.value = value;
//         self.notify();
//     }

//     fn add_waker(&mut self, waker: MutableWaker) {
//         self.wakers.push(waker);
//     }
// }

// #[derive(Clone)]
// pub struct CancelationToken(Rc<AtomicBool>);

// impl CancelationToken {
//     fn new() -> Self {
//         Self(Rc::new(AtomicBool::new(false)))
//     }

//     fn is_canceled(&self) -> bool {
//         self.0.load(std::sync::atomic::Ordering::SeqCst)
//     }

//     pub fn cancel(&self) {
//         self.0.store(true, std::sync::atomic::Ordering::SeqCst)
//     }
// }

// #[derive(Clone)]
// struct MutableWaker {
//     waker: Waker,
//     cancel: CancelationToken,
// }

// impl MutableWaker {
//     fn new(waker: Waker) -> Self {
//         Self {
//             waker,
//             cancel: CancelationToken::new(),
//         }
//     }
// }

// impl<V> Mutable<V> {
//     pub fn new(value: V) -> Self {
//         Self(Rc::new(RefCell::new(MutableState {
//             value,
//             wakers: Vec::new(),
//         })))
//     }

//     pub fn handle(&self) -> MutableHandle<V> {
//         MutableHandle(Rc::downgrade(&self.0))
//     }

//     pub fn set(&mut self, value: V) {
//         let mut state = self.0.borrow_mut();
//         state.set(value);
//     }

//     pub fn modify<F, O>(&mut self, mapper: F) -> O
//     where
//         F: FnOnce(&mut V) -> O,
//     {
//         let mut state = self.0.borrow_mut();
//         let out = mapper(&mut state.value);
//         state.notify();
//         out
//     }

//     pub fn get(&self) -> MutableBorrow<'_, V> {
//         MutableBorrow(self.0.borrow())
//     }
// }

// impl<V: Clone> Mutable<V> {
//     pub fn get_cloned(&self) -> V {
//         self.0.borrow().value.clone()
//     }

//     pub fn stream_cloned(&self) -> MutableStreamCloned<V> {
//         MutableStreamCloned::new(self)
//     }
// }

// // impl<V> Drop for Mutable<V> {
// //     fn drop(&mut self) {
// //         let mut state = self.0.borrow_mut();
// //         // Cancel all listeners.
// //         // for waker in state.wakers.drain(..) {
// //         //     waker
// //         //         .is_canceled
// //         //         .store(true, std::sync::atomic::Ordering::Relaxed);
// //         // }
// //     }
// // }

// pub struct MutableBorrow<'a, V>(std::cell::Ref<'a, MutableState<V>>);

// impl<'a, V> std::ops::Deref for MutableBorrow<'a, V> {
//     type Target = V;

//     fn deref(&self) -> &Self::Target {
//         &self.0.deref().value
//     }
// }

// pub struct MutableHandle<V>(Weak<RefCell<MutableState<V>>>);

// #[derive(Debug)]
// pub struct MutableDroppedError;

// impl std::fmt::Display for MutableDroppedError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Mutable was already dropped")
//     }
// }

// impl std::error::Error for MutableDroppedError {}

// impl<V> MutableHandle<V> {
//     pub fn get_map<O, F: FnOnce(&V) -> O>(&self, mapper: F) -> Result<O, MutableDroppedError> {
//         let state = self.0.upgrade().ok_or(MutableDroppedError)?;
//         let out = mapper(&state.borrow().value);
//         Ok(out)
//     }

//     pub fn set(&self, value: V) -> Result<(), MutableDroppedError> {
//         if let Some(state) = self.0.upgrade() {
//             let mut state = state.borrow_mut();
//             state.set(value);
//         }
//         Ok(())
//     }

//     pub fn modify<F, O>(&mut self, mapper: F) -> Result<O, MutableDroppedError>
//     where
//         F: FnOnce(&mut V) -> O,
//     {
//         if let Some(state) = self.0.upgrade() {
//             let mut state = state.borrow_mut();
//             let out = mapper(&mut state.value);
//             state.notify();
//             Ok(out)
//         } else {
//             Err(MutableDroppedError)
//         }
//     }
// }

// impl<V: Clone> MutableHandle<V> {
//     pub fn get_cloned(&self) -> Result<V, MutableDroppedError> {
//         if let Some(state) = self.0.upgrade() {
//             Ok(state.borrow().value.clone())
//         } else {
//             Err(MutableDroppedError)
//         }
//     }
// }

// pub struct MutableStreamCloned<V> {
//     handle: MutableHandle<V>,
//     cancel: CancelationToken,
//     waker_installed: bool,
// }

// impl<V> MutableStreamCloned<V> {
//     fn new(m: &Mutable<V>) -> Self {

//         Self {
//             handle: m.handle(),
//             cancel: CancelationToken::new(),
//             waker_installed: false,
//         }
//     }
// }

// impl<V> futures::Stream for MutableStreamCloned<V>
// where
//     V: Clone,
// {
//     type Item = V;

//     fn poll_next(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Option<Self::Item>> {
//         let inner = self.get_mut();
//         if let Some(state) = inner.handle.0.upgrade() {
//             if !inner.waker_installed {
//                 let mut state = state.borrow_mut();
//                 state.add_waker(MutableWaker { waker: cx.waker().clone(), cancel: inner.cancel.clone() });
//                 Poll::Ready(Some(state.value.clone()))
//             }  else {
//                 let state = state.borrow();
//                 Poll::Ready(Some(state.value.clone()))
//             }
//         } else {
//             Poll::Ready(None)
//         }
//     }
// }

// pub struct MutableStreamMap<V, O, F: Fn(&V) -> O> {
//     handle: MutableHandle<V>,
//     waker: Rc<Waker>,
//     mapper: F,
// }


// impl<V, O, F> futures::Stream for MutableStreamMap<V, O, F>
// where
//     F: Fn(&V) -> O + std::marker::Unpin,
// {
//     type Item = O;

//     fn poll_next(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Option<Self::Item>> {
//         let inner = self.get_mut();
//         if let Some(state) = inner.handle.0.upgrade() {
//             *Rc::get_mut(&mut inner.waker).unwrap() = cx.waker().clone();
//             let state = state.borrow();
//             let value = (inner.mapper)(&state.value);
//             Poll::Ready(Some(value))
//         } else {
//             Poll::Ready(None)
//         }
//     }
// }
