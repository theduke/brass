use brass::{
    dom::Attr,
    vdom::{div, Ref},
};
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

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

    fn render(&self, _ctx: brass::RenderContext<Self>) -> brass::VNode {
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

fn get_root() -> web_sys::Element {
    let doc = brass::util::document();
    if let Some(elem) = doc.get_element_by_id("testapp") {
        elem.remove();
    }

    let elem = doc.create_element("div").unwrap();
    doc.body().unwrap().append_child(&elem).unwrap();
    elem
}

#[wasm_bindgen_test]
fn refs() {
    tracing_wasm::set_as_global_default();
    brass::boot::<RefComponent>((), get_root());
}
