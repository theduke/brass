use copper::{
    dom::{Attr, Event, Tag},
    vdom::{self, div, DomExtend, TagBuilder},
};
use vdom::{span, tag};

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

pub fn button<M>() -> TagBuilder<M> {
    vdom::button().class("button")
}

pub fn button_medium<M>() -> TagBuilder<M> {
    vdom::button().class("button is-medium")
}

pub fn button_large<M>() -> TagBuilder<M> {
    vdom::button().class("button is-large")
}

pub fn h2<M>() -> TagBuilder<M> {
    vdom::h2().class("title is-2")
}

pub fn h2_with<M, C: DomExtend<M>>(content: C) -> TagBuilder<M> {
    vdom::h2().class("title is-2").and(content)
}

pub fn menu_list<M>() -> TagBuilder<M> {
    vdom::ul().class("menu-list")
}

pub fn icon_fa<M>(icon: &str) -> TagBuilder<M> {
    span()
        .class("icon")
        .attr(Attr::AriaHidden, "true")
        .and(tag(Tag::I).class(icon))
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

pub fn file_input<M>(
    label: &str,
    on_input: fn(web_sys::Event) -> Option<M>,
    disabled: bool,
    multi: bool,
) -> TagBuilder<M>
where
    M: 'static,
{
    let input = vdom::input()
        .class("file-input")
        .attr(Attr::Type, "file")
        .attr_if(disabled, Attr::Disabled, "")
        .attr_if(multi, Attr::Multiple, "")
        .on(Event::Input, on_input);
    let icon = span()
        .class("file-icon")
        .and(vdom::tag(Tag::I).class("fas fa-upload"));
    let inner_label = span().class("file-label").and(label);

    let extra = span().class("file-cta").and(icon).and(inner_label);

    let label = vdom::label().class("file-label").and(input).and(extra);
    div().class("file").and(label)
}

// Panels.

pub fn panel<M>() -> TagBuilder<M> {
    vdom::tag(Tag::Nav).class("panel")
}

pub fn panel_heading<M, C: DomExtend<M>>(content: C) -> TagBuilder<M> {
    vdom::p_with(content).class("panel-heading")
}

pub fn panel_block<M>() -> TagBuilder<M> {
    div().class("panel-block")
}

pub fn panel_icon_fa<M>(icon: &str) -> TagBuilder<M> {
    span()
        .class("panel-icon")
        .and(vdom::tag(Tag::I).class(icon))
}
