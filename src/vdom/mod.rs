pub mod render;

mod node;
pub use node::*;

mod builder;
pub use builder::*;

pub mod event;

use std::rc::Rc;

#[derive(Clone)]
pub enum Func<I, O> {
    Static(fn(I) -> O),
    Dyn(Rc<dyn Fn(I) -> O>),
}

impl<I, O> Func<I, O> {
    pub fn dynamic(f: impl Fn(I) -> O + 'static) -> Self {
        Self::Dyn(Rc::new(f))
    }

    pub fn call(&self, input: I) -> O {
        match self {
            Self::Static(f) => f(input),
            Self::Dyn(f) => (f.as_ref())(input),
        }
    }
}

/// A render fn that can receives the given data as input and renders to a
/// virtual DOM node.
///
/// This is useful for representing dynamic renderers in applications.
pub type Renderer<I> = Func<I, VNode>;

impl<I> Renderer<I> {
    pub fn render(&self, input: I) -> VNode {
        match self {
            Self::Static(f) => f(input),
            Self::Dyn(f) => (f.as_ref())(input),
        }
    }
}

#[derive(Clone)]
pub enum RefFunc<I, O> {
    Static(fn(&I) -> O),
    Dyn(Rc<dyn Fn(&I) -> O>),
}

impl<I, O> RefFunc<I, O> {
    pub fn dynamic(f: impl Fn(&I) -> O + 'static) -> Self {
        Self::Dyn(Rc::new(f))
    }

    pub fn call(&self, input: &I) -> O {
        match self {
            Self::Static(f) => f(input),
            Self::Dyn(f) => (f.as_ref())(input),
        }
    }
}

/// A render fn that can receives the given data as input and renders to a
/// virtual DOM node.
///
/// This is useful for representing dynamic renderers in applications.
pub type RefRenderer<I> = RefFunc<I, VNode>;

impl<I> RefRenderer<I> {
    pub fn render(&self, input: &I) -> VNode {
        match self {
            Self::Static(f) => f(input),
            Self::Dyn(f) => (f.as_ref())(input),
        }
    }
}

/// Wrapper for shared data.
///
/// The primary purpose is to share immutable data between components via props.
///
/// This is just a simple wrapper around [`std::rc::Rc`].
///
/// The inner type **MUST NOT contain any INTERIOR MUTABILITY**.
/// It is not unsafe to have interior mutability, but it may break rendering
/// logic, since [`Shared`] comparisons only compare the memory location, not
/// the actual data.
pub struct Shared<T>(Rc<T>);

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T> AsRef<T> for Shared<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T> std::ops::Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for Shared<T> {}
