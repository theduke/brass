use brass::{util::EventSubscription, vdom::Renderer, Callback, PropComponent};
use wasm_bindgen::JsCast;

pub struct Modal {
    pub render: Renderer<()>,
    pub on_close: Callback<()>,
    pub handle_escape: bool,
}

struct State {
    escape_subscription: Option<EventSubscription>,
    escape_callback: Callback<()>,
}

brass::enable_props!(wrapped Modal => State);

type EscapePressed = ();

impl PropComponent for State {
    type Properties = Modal;
    type Msg = EscapePressed;

    fn init(_props: &Self::Properties, ctx: &mut brass::Context<Self::Msg>) -> Self {
        Self {
            escape_subscription: None,
            escape_callback: ctx.callback(),
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        props: &Self::Properties,
        _ctx: &mut brass::Context<Self::Msg>,
    ) {
        match msg {
            () => {
                props.on_close.send(());
            }
        }
    }

    fn render(
        &self,
        props: &Self::Properties,
        _ctx: &mut brass::RenderContext<brass::PropWrapper<Self>>,
    ) -> brass::VNode {
        super::modal(props.render.call(()), &props.on_close.clone()).build()
    }

    fn on_render(&mut self, props: &Self::Properties, first_render: bool) {
        if first_render && props.handle_escape {
            let doc = brass::util::document();
            let guard = brass::util::EventSubscription::subscribe_filtered(
                doc.unchecked_into(),
                brass::dom::Event::KeyDown,
                self.escape_callback.clone(),
                |ev: web_sys::KeyboardEvent| {
                    if ev.code() == "Escape" {
                        Some(())
                    } else {
                        None
                    }
                },
            );
            self.escape_subscription = Some(guard);
        }
    }
}
