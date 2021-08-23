mod dropdown;

use std::{collections::HashSet, rc::Rc};

use wasm_bindgen::JsCast;

pub use dropdown::Dropdown;

use brass::{
    dom::{Attr, Event, Tag},
    vdom::{self, div, s, DomExtend, EventCallback, Render, TagBuilder},
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
    div().class(s("box"))
}

pub fn navbar_main() -> TagBuilder {
    div()
        .class(s("navbar"))
        .attr(Attr::Role, s("navigation"))
        .attr(Attr::AriaLabel, s("main-navigation"))
}

pub fn navbar_brand() -> TagBuilder {
    div().class(s("navbar-brand"))
}

pub fn navbar_menu() -> TagBuilder {
    div().class(s("navbar-menu"))
}

pub fn navbar_start() -> TagBuilder {
    div().class(s("navbar-start"))
}

pub fn navbar_item_with(content: impl DomExtend) -> TagBuilder {
    div().class(s("navbar-item")).and(content)
}

// Forms.

pub fn field() -> TagBuilder {
    div().class(s("field"))
}

pub fn field_with<C: DomExtend>(content: C) -> TagBuilder {
    div().class(s("field")).and(content)
}

pub fn label() -> TagBuilder {
    vdom::label().class(s("label"))
}

pub fn label_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::label().class(s("label")).and(content)
}

pub fn control() -> TagBuilder {
    div().class(s("control"))
}

pub fn control_with<C: DomExtend>(content: C) -> TagBuilder {
    div().class(s("control")).and(content)
}

pub fn input() -> TagBuilder {
    vdom::input().class(s("input"))
}

pub fn input_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::input().class(s("input")).and(content)
}

pub fn field_help() -> TagBuilder {
    vdom::p().class(s("help"))
}

pub fn field_help_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::p().class(s("help")).and(content)
}

pub fn button() -> TagBuilder {
    vdom::button().class(s("button"))
}

pub fn button_small() -> TagBuilder {
    vdom::button().class(s("button is-small"))
}

pub fn button_medium() -> TagBuilder {
    vdom::button().class(s("button is-medium"))
}

pub fn button_large() -> TagBuilder {
    vdom::button().class(s("button is-large"))
}

pub fn buttons() -> TagBuilder {
    vdom::div().class(s("buttons"))
}

pub fn h2() -> TagBuilder {
    vdom::h2().class(s("title is-2"))
}

pub fn h2_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::h2().class(s("title is-2")).and(content)
}

pub fn h3() -> TagBuilder {
    vdom::h3().class(s("title is-3"))
}

pub fn h3_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::h3().class(s("title is-3")).and(content)
}

pub fn h4() -> TagBuilder {
    vdom::h4().class(s("title is-4"))
}

pub fn h4_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::h4().class(s("title is-4")).and(content)
}

pub fn h5() -> TagBuilder {
    vdom::h5().class(s("title is-5"))
}

pub fn h5_with<C: DomExtend>(content: C) -> TagBuilder {
    vdom::h5().class(s("title is-5")).and(content)
}

pub fn menu_list() -> TagBuilder {
    vdom::ul().class(s("menu-list"))
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
    vdom::table().class(s("table"))
}

pub fn icon_fa(icon: impl Into<Str>) -> TagBuilder {
    span()
        .class(s("icon"))
        .attr(Attr::AriaHidden, "true")
        .and(tag(Tag::I).class(icon))
}

pub fn modal<C: DomExtend>(content: C, on_close: Callback<()>) -> TagBuilder {
    let on_close_ev = EventCallback::callback(|_| (), on_close);
    let bg = div()
        .class(s("modal-background"))
        .on(Event::Click, on_close_ev.clone());

    let inner = div().class(s("modal-content")).and(content);

    let close = button()
        .class(s("modal-close is-large"))
        .attr(Attr::AriaLabel, "close")
        .on(Event::Click, on_close_ev.clone());

    div()
        .class(s("modal is-active"))
        .and(bg)
        .and(inner)
        .and(close)
}

pub fn file_input(label: &str, on_input: EventCallback, disabled: bool, multi: bool) -> TagBuilder {
    let input = vdom::input()
        .class(s("file-input"))
        .attr(Attr::Type, "file")
        .attr_if(disabled, Attr::Disabled, "")
        .attr_if(multi, Attr::Multiple, "")
        .on(Event::Input, on_input);
    let icon = span()
        .class(s("file-icon"))
        .and(vdom::tag(Tag::I).class(s("fas fa-upload")));
    let inner_label = span().class(s("file-label")).and(label);

    let extra = span().class(s("file-cta")).and(icon).and(inner_label);

    let label = vdom::label().class(s("file-label")).and(input).and(extra);
    div().class(s("file")).and(label)
}

// Panels.

pub fn panel() -> TagBuilder {
    vdom::tag(Tag::Nav).class(s("panel"))
}

pub fn panel_heading<C: DomExtend>(content: C) -> TagBuilder {
    vdom::p_with(content).class(s("panel-heading"))
}

pub fn panel_block() -> TagBuilder {
    div().class(s("panel-block"))
}

pub fn panel_icon_fa(icon: &str) -> TagBuilder {
    span()
        .class(s("panel-icon"))
        .and(vdom::tag(Tag::I).class(icon))
}

// Cards.

pub fn card() -> TagBuilder {
    div().class(s("card"))
}

pub fn card_header() -> TagBuilder {
    vdom::header().class(s("card-header"))
}

pub fn card_header_title(content: impl DomExtend) -> TagBuilder {
    vdom::p().class(s("card-header-title")).and(content)
}

pub fn card_content() -> TagBuilder {
    div().class(s("card-content"))
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
                    .class(s("field-label is-normal"))
                    .and(label_with(self.label)),
                div().class(s("field-body")).and(
                    field()
                        .and(div().class(s("control")).and(self.control))
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
    pub empty_option_label: Option<Str>,
    pub options: Rc<Vec<SelectOption<T>>>,
    pub on_select: Callback<Option<T>>,
}

impl<T: PartialEq + Eq + Clone> Render for Select<T> {
    fn render(self) -> VNode {
        let options = self.options.iter().enumerate().map(|(index, opt)| {
            let selected = self.value.as_ref() == Some(&opt.value);
            vdom::option()
                .attr(Attr::Value, index.to_string())
                .attr_toggle_if(selected, Attr::Selected)
                .and(opt.label.clone())
        });
        let empty_option = self.empty_option_label.as_ref().map(|label| {
            vdom::option()
                .attr(Attr::Value, s(""))
                .attr_toggle_if(self.value.is_none(), Attr::Selected)
                .and(label.clone())
        });

        let opts = self.options.clone();
        // TODO: this clone is redundant.
        let callback = self.on_select.clone();
        let select = vdom::select().and(empty_option).and_iter(options).on(
            Event::Change,
            EventCallback::Closure(std::rc::Rc::new(move |ev: web_sys::Event| {
                let value = ev
                    .target()?
                    .dyn_ref::<web_sys::HtmlSelectElement>()?
                    .value();

                if value == "" {
                    callback.send(None);
                } else {
                    let index = value.parse::<usize>().ok()?;
                    let value = opts.get(index)?.value.clone();
                    callback.send(Some(value));
                }
                None
            })),
        );

        div().class(s("select")).and(select).build()
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

        div().class(s("tags")).and_iter(options).build()
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
            .class(s("file-input"))
            .attr(Attr::Type, "file")
            .attr_toggle_if(self.multi, Attr::Multiple)
            .on(Event::Change, self.on_change);
        let cta = vdom::span().class(s("file-cta")).and((
            vdom::span()
                .class(s("file-icon"))
                .and(vdom::i().class(s("fas fa-upload"))),
            vdom::span().class(s("file-label")).and(self.label),
        ));
        let label = vdom::label().class(s("file-label")).and((input, cta));
        let n = div().class(s("file")).and(label);

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
        let lbl = vdom::label().class(s("checkbox")).and(inp).and(self.label);
        let ctrl = control().and(lbl);
        field().and(ctrl)
    }
}

impl DomExtend for Checkbox {
    fn extend(self, parent: &mut TagBuilder) {
        parent.add_child(self.render());
    }
}
