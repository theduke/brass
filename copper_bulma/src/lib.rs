use copper::{
    dom::{Attr, Event, Tag},
    vdom::{self, div, DomExtend, TagBuilder},
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

pub fn h3<M>() -> TagBuilder<M> {
    vdom::h3().class("title is-3")
}

pub fn h3_with<M, C: DomExtend<M>>(content: C) -> TagBuilder<M> {
    vdom::h3().class("title is-3").and(content)
}

pub fn h4<M>() -> TagBuilder<M> {
    vdom::h4().class("title is-4")
}

pub fn h4_with<M, C: DomExtend<M>>(content: C) -> TagBuilder<M> {
    vdom::h4().class("title is-4").and(content)
}

pub fn h5<M>() -> TagBuilder<M> {
    vdom::h5().class("title is-5")
}

pub fn h5_with<M, C: DomExtend<M>>(content: C) -> TagBuilder<M> {
    vdom::h5().class("title is-5").and(content)
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

pub struct Help<T> {
    pub message: T,
    pub color: Color,
}

impl<M, T> DomExtend<M> for Help<T>
where
    T: DomExtend<M>,
{
    fn extend(self, parent: &mut TagBuilder<M>) {
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
    pub fn render<M>(self) -> TagBuilder<M>
    where
        C: DomExtend<M>,
    {
        field()
            .and(label_with(self.label))
            .and(control_with(self.control))
            .and(self.help)
    }
}

impl<M, C> DomExtend<M> for Field<C>
where
    C: DomExtend<M> + 'static,
    M: 'static,
{
    fn extend(self, parent: &mut TagBuilder<M>) {
        parent.add_child(self.render());
    }
}

pub struct Input<F> {
    pub _type: &'static str,
    pub color: Color,
    pub placeholder: Option<String>,
    pub value: String,
    pub on_input: F,
}

impl<F, M> DomExtend<M> for Input<F>
where
    F: Fn(String) -> Option<M> + Clone + 'static,
    M: 'static,
{
    fn extend(self, parent: &mut TagBuilder<M>) {
        let on_input = self.on_input;
        let mut inp = input()
            .and_class(self.color.as_class())
            .attr(Attr::Value, self.value)
            .on_captured(Event::Input, move |ev| {
                copper::input_event_value(ev).and_then(|v| on_input(v))
            });

        if let Some(placeholder) = self.placeholder {
            inp.add_attr(Attr::Placeholder, placeholder);
        }

        parent.add_child(inp);
    }
}

pub struct Checkbox<F> {
    pub color: Color,
    pub label: String,
    pub value: bool,
    pub on_input: F,
}

impl<F, M> Checkbox<F>
where
    F: Fn(bool) -> Option<M> + Clone + 'static,
    M: 'static,
{
    pub fn render(self) -> TagBuilder<M> {
        let on_input = self.on_input;

        let inp = vdom::input()
            .attr(Attr::Type, "checkbox")
            .on_captured(Event::Input, move |ev| {
                copper::input_event_checkbox_value(ev).and_then(|flag| on_input(flag))
            });
        let lbl = vdom::label().class("checkbox").and(inp).and(self.label);
        let ctrl = control().and(lbl);
        field().and(ctrl)
    }
}

impl<F, M> DomExtend<M> for Checkbox<F>
where
    F: Fn(bool) -> Option<M> + Clone + 'static,
    M: 'static,
{
    fn extend(self, parent: &mut TagBuilder<M>) {
        parent.add_child(self.render());
    }
}
