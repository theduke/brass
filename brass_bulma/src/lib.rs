use brass::{
    dom::{Attr, Event, Tag},
    vdom::{self, div, DomExtend, EventCallback, TagBuilder},
};
use vdom::{span, tag};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Default,
    Primary,
    Link,
    Info,
    Success,
    Warning,
    Danger,
}

impl Color {
    pub fn as_class(self) -> &'static str {
        match self {
            Color::Default => "",
            Color::Primary => "is-primary",
            Color::Link => "is-link",
            Color::Info => "is-info",
            Color::Success => "is-success",
            Color::Warning => "is-warning",
            Color::Danger => "is-danger",
        }
    }
}

pub fn box_() -> TagBuilder {
    div().class("box")
}

pub fn navbar_main() -> TagBuilder {
    div()
        .class("navbar")
        .attr(Attr::Role, "navigation")
        .attr(Attr::AriaLabel, "main-navigation")
}

pub fn navbar_brand() -> TagBuilder {
    div().class("navbar-brand")
}

pub fn navbar_menu() -> TagBuilder {
    div().class("navbar-menu")
}

pub fn navbar_start() -> TagBuilder {
    div().class("navbar-start")
}

pub fn navbar_item_with(content: impl DomExtend) -> TagBuilder {
    div().class("navbar-item").and(content)
}

// Forms.

pub fn field() -> TagBuilder {
    div().class("field")
}

pub fn field_with<C: DomExtend>(content: C) -> TagBuilder {
    div().class("field").and(content)
}

pub fn label() -> TagBuilder {
    vdom::label().class("label")
}

pub fn label_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::label().class("label").and(content)
}

pub fn control() -> TagBuilder {
    div().class("control")
}

pub fn control_with<C: DomExtend>(content: C) -> TagBuilder {
    div().class("control").and(content)
}

pub fn input() -> TagBuilder {
    vdom::input().class("input")
}

pub fn input_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::input().class("input").and(content)
}

pub fn field_help() -> TagBuilder {
    vdom::p().class("help")
}

pub fn field_help_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::p().class("help").and(content)
}

pub fn button() -> TagBuilder {
    vdom::button().class("button")
}

pub fn button_medium() -> TagBuilder {
    vdom::button().class("button is-medium")
}

pub fn button_large() -> TagBuilder {
    vdom::button().class("button is-large")
}

pub fn h2() -> TagBuilder {
    vdom::h2().class("title is-2")
}

pub fn h2_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::h2().class("title is-2").and(content)
}

pub fn h3() -> TagBuilder {
    vdom::h3().class("title is-3")
}

pub fn h3_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::h3().class("title is-3").and(content)
}

pub fn h4() -> TagBuilder {
    vdom::h4().class("title is-4")
}

pub fn h4_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::h4().class("title is-4").and(content)
}

pub fn h5() -> TagBuilder {
    vdom::h5().class("title is-5")
}

pub fn h5_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::h5().class("title is-5").and(content)
}

pub fn menu_list() -> TagBuilder {
    vdom::ul().class("menu-list")
}

pub fn icon_fa(icon: &str) -> TagBuilder {
    span()
        .class("icon")
        .attr(Attr::AriaHidden, "true")
        .and(tag(Tag::I).class(icon))
}

pub fn modal<C: DomExtend>(content: C, on_close: EventCallback) -> TagBuilder {
    let on_close2 = on_close.clone();
    let bg = div()
        .class("modal-background")
        .on(Event::Click, on_close.clone());

    let inner = div().class("modal-content").and(content);

    let close = button()
        .class("modal-close is-large")
        .attr(Attr::AriaLabel, "close")
        .on(Event::Click, on_close2);

    div().class("modal is-active").and(bg).and(inner).and(close)
}

pub fn file_input(label: &str, on_input: EventCallback, disabled: bool, multi: bool) -> TagBuilder {
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

pub fn panel() -> TagBuilder {
    vdom::tag(Tag::Nav).class("panel")
}

pub fn panel_heading<C: DomExtend>(content: C) -> TagBuilder {
    vdom::p_with(content).class("panel-heading")
}

pub fn panel_block() -> TagBuilder {
    div().class("panel-block")
}

pub fn panel_icon_fa(icon: &str) -> TagBuilder {
    span()
        .class("panel-icon")
        .and(vdom::tag(Tag::I).class(icon))
}

pub struct Help<T> {
    pub message: T,
    pub color: Color,
}

impl<T> DomExtend for Help<T>
where
    T: DomExtend,
{
    fn extend(self, parent: &mut TagBuilder) {
        let content = field_help()
            .and_class(self.color.as_class())
            .and(self.message);
        parent.add_child(content);
    }
}

pub struct Field<C> {
    pub label: String,
    pub help: Option<Help<String>>,
    pub control: C,
}

impl<C> Field<C> {
    pub fn render(self) -> TagBuilder
    where
        C: DomExtend,
    {
        field()
            .and(label_with(self.label))
            .and(control_with(self.control))
            .and(self.help)
    }
}

impl<C> DomExtend for Field<C>
where
    C: DomExtend + 'static,
{
    fn extend(self, parent: &mut TagBuilder) {
        parent.add_child(self.render());
    }
}

pub struct Input {
    pub _type: &'static str,
    pub color: Color,
    pub placeholder: Option<String>,
    pub value: String,
    pub on_input: EventCallback,
}

impl DomExtend for Input {
    fn extend(self, parent: &mut TagBuilder) {
        let mut inp = input()
            .and_class(self.color.as_class())
            .attr(Attr::Value, self.value)
            .on(Event::Input, self.on_input);

        if let Some(placeholder) = self.placeholder {
            inp.add_attr(Attr::Placeholder, placeholder);
        }

        parent.add_child(inp);
    }
}

pub struct Checkbox {
    pub color: Color,
    pub label: String,
    pub value: bool,
    pub on_input: EventCallback,
}

impl Checkbox {
    pub fn render(self) -> TagBuilder {
        let inp = vdom::input()
            .attr(Attr::Type, "checkbox")
            .on(Event::Input, self.on_input);
        let lbl = vdom::label().class("checkbox").and(inp).and(self.label);
        let ctrl = control().and(lbl);
        field().and(ctrl)
    }
}

impl DomExtend for Checkbox {
    fn extend(self, parent: &mut TagBuilder) {
        parent.add_child(self.render());
    }
}
