mod dropdown;

use std::{collections::HashSet, rc::Rc};

use wasm_bindgen::JsCast;

pub use dropdown::Dropdown;

use brass::{
    dom::{Attr, Event, Tag},
    vdom::{self, div, DomExtend, EventCallback, Render, TagBuilder},
    Callback, Str, VNode,
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

pub fn button_small() -> TagBuilder {
    vdom::button().class("button is-small")
}

pub fn button_medium() -> TagBuilder {
    vdom::button().class("button is-medium")
}

pub fn button_large() -> TagBuilder {
    vdom::button().class("button is-large")
}

pub fn buttons() -> TagBuilder {
    vdom::div().class("buttons")
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

pub fn notification(color: Color, content: impl DomExtend) -> TagBuilder {
    vdom::p_with(content).class(&format!("notification {}", color.as_class()))
}

pub fn notification_success(content: impl DomExtend) -> TagBuilder {
    notification(Color::Success, content)
}

pub fn notification_warning(content: impl DomExtend) -> TagBuilder {
    notification(Color::Warning, content)
}

pub fn notification_error(content: impl DomExtend) -> TagBuilder {
    notification(Color::Danger, content)
}

#[inline]
pub fn table() -> TagBuilder {
    vdom::table().class("table")
}

pub fn icon_fa(icon: impl Into<Str>) -> TagBuilder {
    span()
        .class("icon")
        .attr(Attr::AriaHidden, "true")
        .and(tag(Tag::I).class(icon))
}

pub fn modal<C: DomExtend>(content: C, on_close: Callback<()>) -> TagBuilder {
    let on_close_ev = EventCallback::callback(|_| (), on_close);
    let bg = div()
        .class("modal-background")
        .on(Event::Click, on_close_ev.clone());

    let inner = div().class("modal-content").and(content);

    let close = button()
        .class("modal-close is-large")
        .attr(Attr::AriaLabel, "close")
        .on(Event::Click, on_close_ev.clone());

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

// Cards.

pub fn card() -> TagBuilder {
    div().class("card")
}

pub fn card_header() -> TagBuilder {
    vdom::header().class("card-header")
}

pub fn card_header_title(content: impl DomExtend) -> TagBuilder {
    vdom::p().class("card-header-title").and(content)
}

pub fn card_content() -> TagBuilder {
    div().class("card-content")
}

#[derive(Debug)]
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
    pub label: Str,
    pub help: Option<Help<Str>>,
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

pub struct FieldHorizontal<C> {
    pub label: Str,
    pub help: Option<Help<VNode>>,
    pub control: C,
}

impl<C: Render> Render for FieldHorizontal<C> {
    fn render(self) -> VNode
    where
        C: DomExtend,
    {
        field()
            .and_class("is-horizontal")
            .and((
                div()
                    .class("field-label is-normal")
                    .and(label_with(self.label)),
                div().class("field-body").and(
                    field()
                        .and(div().class("control").and(self.control))
                        .and_opt(self.help),
                ),
            ))
            .build()
    }
}

pub struct Input {
    pub _type: &'static str,
    pub color: Color,
    pub placeholder: Option<Str>,
    pub value: Str,
    pub on_input: EventCallback,
}

impl Render for Input {
    fn render(self) -> VNode {
        let mut inp = input()
            .and_class(self.color.as_class())
            .attr(Attr::Type, self._type)
            .attr(Attr::Value, self.value)
            .on(Event::Input, self.on_input);

        if let Some(placeholder) = self.placeholder {
            inp.add_attr(Attr::Placeholder, placeholder);
        }

        inp.build()
    }
}

pub struct Textarea {
    pub color: Color,
    pub placeholder: Option<Str>,
    pub value: Str,
    pub on_input: EventCallback,
    pub on_keydown: Option<EventCallback>,
    pub style_raw: Option<Str>,
}

impl Render for Textarea {
    fn render(self) -> VNode {
        let mut area = tag(Tag::TextArea)
            .and_class(self.color.as_class())
            .and_class("textarea")
            .attr(Attr::Value, self.value)
            .on(Event::Input, self.on_input);

        if let Some(handler) = self.on_keydown {
            area = area.on(Event::KeyDown, handler);
        }

        if let Some(placeholder) = self.placeholder {
            area.add_attr(Attr::Placeholder, placeholder);
        }

        if let Some(style) = self.style_raw {
            area = area.style_raw(style);
        }

        area.build()
    }
}

#[derive(Clone, Debug)]
pub struct SelectOption<T> {
    pub value: T,
    pub label: Str,
}

pub struct Select<T: 'static> {
    pub value: Option<T>,
    pub options: Rc<Vec<SelectOption<T>>>,
    pub on_select: Callback<Option<T>>,
}

impl<T: PartialEq + Eq + Clone> Render for Select<T> {
    fn render(self) -> VNode {
        let options = self.options.iter().enumerate().map(|(index, opt)| {
            let selected = self.value.as_ref() == Some(&opt.value);
            vdom::option()
                .attr(Attr::Value, index.to_string())
                .attr_toggle_if(selected, Attr::Checked)
                .and(opt.label.clone())
        });

        let opts = self.options.clone();
        // TODO: this clone is redundant.
        let callback = self.on_select.clone();
        let select = vdom::select().and_iter(options).on(
            Event::Change,
            EventCallback::Closure(std::rc::Rc::new(move |ev: web_sys::Event| {
                let index = ev
                    .target()?
                    .dyn_ref::<web_sys::HtmlSelectElement>()?
                    .value()
                    .parse::<usize>()
                    .ok()?;
                let value = opts.get(index)?.value.clone();
                callback.send(Some(value));

                None
            })),
        );

        div().class("select").and(select).build()
    }
}

pub struct TagSelect<'a, T: 'static> {
    pub options: &'a [SelectOption<T>],
    pub selected: &'a HashSet<T>,
    pub on_select: Callback<T>,
}

impl<'a, T: PartialEq + Eq + Clone + std::hash::Hash> Render for TagSelect<'a, T> {
    fn render(self) -> VNode {
        let options = self.options.iter().enumerate().map(|(index, opt)| {
            let selected = self.selected.contains(&opt.value);

            let class = if selected {
                "tag is-primary is-clickable is-unselectable"
            } else {
                "tag is-clickable is-unselectable"
            };

            // TODO: use a single event handler that reads the index from
            // a data="" attribute to improve performance.
            let on_select = self.on_select.clone();
            let value = opt.value.clone();

            span()
                .class(class)
                .attr(Attr::Data, index.to_string())
                .and(opt.label.clone())
                .on_click(EventCallback::Closure(Rc::new(move |_| {
                    on_select.send(value.clone());
                    None
                })))
        });

        div().class("tags").and_iter(options).build()
    }
}

pub struct FileInput {
    pub label: Str,
    pub multi: bool,
    pub on_change: EventCallback,
}

impl DomExtend for FileInput {
    fn extend(self, parent: &mut TagBuilder) {
        let input = vdom::input()
            .class("file-input")
            .attr(Attr::Type, "file")
            .attr_toggle_if(self.multi, Attr::Multiple)
            .on(Event::Change, self.on_change);
        let cta = vdom::span().class("file-cta").and((
            vdom::span()
                .class("file-icon")
                .and(vdom::i().class("fas fa-upload")),
            vdom::span().class("file-label").and(self.label),
        ));
        let label = vdom::label().class("file-label").and((input, cta));
        let n = div().class("file").and(label);

        parent.add_child(n);
    }
}

pub struct Checkbox {
    pub color: Color,
    pub label: Str,
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
