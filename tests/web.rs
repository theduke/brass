use brass::{
    dom::Attr,
    vdom::{self, div, event::ClickEvent, Ref},
    Callback,
};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn get_root() -> web_sys::Element {
    let doc = brass::util::document();
    if let Some(elem) = doc.get_element_by_id("testapp") {
        elem.remove();
    }

    let elem = doc.create_element("div").unwrap();
    doc.body().unwrap().append_child(&elem).unwrap();
    elem
}

// Refs.

struct RefComponent {
    vref: Ref,
}

impl brass::Component for RefComponent {
    type Properties = ();
    type Msg = ();

    fn init(_props: Self::Properties, _ctx: &mut brass::Context<Self::Msg>) -> Self {
        Self { vref: Ref::new() }
    }

    fn update(&mut self, _msg: Self::Msg, _ctxx: &mut brass::Context<Self::Msg>) {}

    fn render(&self, _ctx: &mut brass::RenderContext<Self>) -> brass::VNode {
        div().attr(Attr::Id, "refcontainer").build_ref(&self.vref)
    }

    fn on_property_change(
        &mut self,
        _props: Self::Properties,
        _ctx: &mut brass::Context<Self::Msg>,
    ) -> brass::ShouldRender {
        false
    }

    fn on_render(&mut self, _first_render: bool) {
        let elem = self.vref.get().expect("Could not obtain reference");
        assert_eq!(elem.id(), "refcontainer");

        let real_elem = brass::util::document()
            .get_element_by_id("refcontainer")
            .unwrap();
        assert_eq!(elem, real_elem);
    }
}

#[wasm_bindgen_test::wasm_bindgen_test]
fn test_refs() {
    tracing_wasm::set_as_global_default();
    brass::boot::<RefComponent>((), get_root());
}

// Callbacks.

pub struct CallbackComp {
    count: u64,
    callback: Callback<u64>,
}

impl brass::Component for CallbackComp {
    type Properties = ();

    type Msg = u64;

    fn init(_props: Self::Properties, ctx: &mut brass::Context<Self::Msg>) -> Self {
        Self {
            count: 0,
            callback: ctx.callback(),
        }
    }

    fn update(&mut self, msg: Self::Msg, _ctx: &mut brass::Context<Self::Msg>) {
        self.count += msg;
    }

    fn render(&self, _ctx: &mut brass::RenderContext<Self>) -> brass::VNode {
        div()
            .and((
                div().class("callback-test").and(self.count.to_string()),
                vdom::button()
                    .class("callback-test-trigger")
                    .on_callback(|_: ClickEvent| 1, &self.callback),
            ))
            .build()
    }

    fn on_property_change(
        &mut self,
        _props: Self::Properties,
        _ctx: &mut brass::Context<Self::Msg>,
    ) -> brass::ShouldRender {
        true
    }
}

#[wasm_bindgen_test::wasm_bindgen_test]
fn test_callback() {
    brass::boot::<CallbackComp>((), get_root());
    let btn =
        brass::util::query_selector_as::<web_sys::HtmlElement>(".callback-test-trigger").unwrap();
    btn.click();
    btn.click();
}
