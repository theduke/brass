use copper::{
    dom::{Attr, Event},
    vdom::{self, div, DomExtend, TagBuilder},
};
use vdom::button;

pub fn box_<M>() -> TagBuilder<M> {
    div().class("box")
}

pub fn navbar_main<M>() -> TagBuilder<M> {
    div()
        .class("navbar")
        .attr(Attr::Role, "navigation")
        .attr(Attr::AriaLabel, "main-navigation")
}

pub fn navbar_brand<M>() -> TagBuilder<M> {
    div().class("navbar-brand")
}

pub fn navbar_menu<M>() -> TagBuilder<M> {
    div().class("navbar-menu")
}

pub fn navbar_start<M>() -> TagBuilder<M> {
    div().class("navbar-start")
}

pub fn navbar_item_with<M>(content: impl DomExtend<M>) -> TagBuilder<M> {
    div().class("navbar-item").and(content)
}

// Forms.

pub fn field<M>() -> TagBuilder<M> {
    div().class("field")
}

pub fn field_with<M, C: DomExtend<M>>(content: C) -> TagBuilder<M> {
    div().class("field").and(content)
}

pub fn label<M>() -> TagBuilder<M> {
    vdom::label().class("label")
}

pub fn label_with<M, C: DomExtend<M>>(content: C) -> TagBuilder<M> {
    vdom::label().class("label").and(content)
}

pub fn control<M>() -> TagBuilder<M> {
    div().class("control")
}

pub fn control_with<M, C: DomExtend<M>>(content: C) -> TagBuilder<M> {
    div().class("control").and(content)
}

pub fn input<M>() -> TagBuilder<M> {
    vdom::input().class("input")
}

pub fn input_with<M, C: DomExtend<M>>(content: C) -> TagBuilder<M> {
    vdom::input().class("input").and(content)
}

pub fn field_help<M>() -> TagBuilder<M> {
    vdom::p().class("help")
}

pub fn field_help_with<M, C: DomExtend<M>>(content: C) -> TagBuilder<M> {
    vdom::p().class("help").and(content)
}

pub fn modal<M: Clone + 'static, C: DomExtend<M>>(content: C, on_close: M) -> TagBuilder<M> {
    let on_close2 = on_close.clone();
    let bg = div()
        .class("modal-background")
        .on_captured(Event::Click, move |_| Some(on_close.clone()));

    let inner = div().class("modal-content").and(content);

    let close = button()
        .class("modal-close is-large")
        .attr(Attr::AriaLabel, "close")
        .on_captured(Event::Click, move |_| Some(on_close2.clone()));

    div().class("modal is-active").and(bg).and(inner).and(close)
}
