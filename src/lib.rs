#![recursion_limit = "512"]

mod any;

mod app;
pub mod dom;
pub mod vdom;

pub mod util;

pub use app::{boot, Callback, Component, Context, EffectGuard, RenderContext, ShouldRender};

pub use vdom::VNode;

/// Enable the properties of a Component to be used when building the virtual
/// dom.
///
/// ```rust
/// struct Props {}
/// struct MyComponent {}
///
/// impl brass::Component for MyComponent {
///     ...
/// }
///
/// enable_props!(Props => MyComponent)
///
/// ```
#[macro_export]
macro_rules! enable_props {
    ($prop:ty => $comp:ty) => {
        impl brass::vdom::DomExtend for $prop {
            fn extend(self, parent: &mut brass::vdom::TagBuilder) {
                let x = <$comp as brass::Component>::build(self);
                parent.add_child(x);
            }
        }
    };
}

#[macro_export]
macro_rules! rsx {
    ( $tag:ident $($extra:tt)* ) => {
        rsx!(
            !
            (
                $crate::vdom::TagBuilder::new($crate::dom::Tag::$tag)
            )
            $($extra)*
        )
    };

    (! ( $($code:tt)* ) $attr:ident = $value:literal $( $extra:tt )* ) => {
        rsx!(
            !
            ( $( $code )* .attr($crate::dom::Attr::$attr, $value) )
            $( $extra )*
        )
    };

    (! ( $($code:tt)* ) ) => {
        $($code)*
    };
}
