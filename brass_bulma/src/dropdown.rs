use brass::{
    vdom::{div, DomExtend, EventCallback},
    VNode,
};

pub type DropdownContentRenderer = Box<dyn Fn() -> VNode>;

pub struct Dropdown<T, C> {
    pub trigger: T,
    pub content: C,
    pub is_active: bool,
    pub is_hoverable: bool,
    pub on_toggle: EventCallback,
}

impl<T: DomExtend, C: DomExtend> DomExtend for Dropdown<T, C> {
    fn extend(self, parent: &mut brass::vdom::TagBuilder) {
        let trigger_btn = super::button().and(self.trigger).on_click(self.on_toggle);

        let trigger = div().class("dropdown-trigger").and(trigger_btn);

        let content = div()
            .class("dropdown-menu")
            .and(div().class("dropdown-content").and(self.content));

        let elem = div()
            .class("dropdown")
            .and_class_if(self.is_active, "is-active")
            .and_class_if(self.is_hoverable, "is-hoverable")
            .and((trigger, content));

        parent.add_child(elem);
    }
}

pub struct DropdownProps {
    pub trigger: String,
    pub content: DropdownContentRenderer,
}

pub struct DropdownComponent {
    trigger: String,
    content: DropdownContentRenderer,
    active: bool,
}

brass::enable_props!(DropdownProps => DropdownComponent);

pub enum Msg {
    Toggle,
}

impl brass::Component for DropdownComponent {
    type Properties = DropdownProps;
    type Msg = Msg;

    fn init(props: Self::Properties, _ctx: &mut brass::Context<Self::Msg>) -> Self {
        Self {
            trigger: props.trigger,
            content: props.content,
            active: false,
        }
    }

    fn update(&mut self, msg: Self::Msg, _ctx: &mut brass::Context<Self::Msg>) {
        match msg {
            Msg::Toggle => {
                self.active = !self.active;
            }
        }
    }

    fn render(&self, ctx: &mut brass::RenderContext<Self>) -> VNode {
        let trigger_btn = super::button()
            .and((self.trigger.clone(), super::icon_fa("fas fa-angle-down")))
            .on_click(ctx.on_simple(|| Msg::Toggle));

        let trigger = div().class("dropdown-trigger").and(trigger_btn);

        let content = div()
            .class("dropdown-menu")
            .and(div().class("dropdown-content").and((self.content)()));

        div()
            .class("dropdown")
            .and_class_if(self.active, "is-active")
            .and((trigger, content))
            .build()
    }

    fn on_property_change(
        &mut self,
        props: Self::Properties,
        _ctx: &mut brass::Context<Self::Msg>,
    ) -> brass::ShouldRender {
        self.content = props.content;
        self.trigger = props.trigger;
        true
    }
}
