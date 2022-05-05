wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
use futures_signals::{signal::Mutable, signal_vec::MutableVec};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::wasm_bindgen_test;

use brass::{
    dom::builder::{div, span},
    view,
};

fn get_root() -> web_sys::Element {
    let doc = brass::web::window().document().unwrap();
    if let Some(elem) = doc.get_element_by_id("testapp") {
        elem.remove();
    }

    let elem = doc.create_element("div").unwrap();
    doc.body().unwrap().append_child(&elem).unwrap();
    elem
}

fn elem_by_id(id: &str) -> web_sys::Element {
    brass::web::window()
        .document()
        .unwrap()
        .get_element_by_id(id)
        .unwrap()
}

async fn tick() {
    let promise = js_sys::Promise::resolve(&JsValue::NULL);
    JsFuture::from(promise).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_signal() {
    let mutable = Mutable::new("hello".to_string());

    let sig = mutable.signal_ref(|v| div().and(v));

    brass::launch(get_root(), || {
        div().attr(brass::dom::Attr::Id, "test-signal").signal(sig)
    });

    // TimeoutFuture::new(Duration::from_millis(10)).await;
    tick().await;

    let elem = elem_by_id("test-signal");
    assert_eq!(elem.inner_html(), "<div>hello</div>");

    mutable.set("v2".to_string());
    tick().await;
    assert_eq!(elem.inner_html(), "<div>v2</div>");
}

#[wasm_bindgen_test]
async fn test_signal_vec_view() {
    let mvec = MutableVec::<&'static str>::new();

    let sig = mvec.signal_vec_cloned();

    brass::launch(get_root(), || {
        div()
            .attr(brass::dom::Attr::Id, "test_signal_vec_view")
            .signal_vec(sig, |x| span().and(*x))
    });

    let elem = elem_by_id("test_signal_vec_view");

    tick().await;
    assert_eq!(elem.inner_html(), "<!---->");

    mvec.lock_mut().push("a");
    tick().await;
    assert_eq!(elem.inner_html(), "<span>a</span><!---->");

    mvec.lock_mut().push("b");
    mvec.lock_mut().push("c");
    tick().await;
    assert_eq!(
        elem.inner_html(),
        "<span>a</span><span>b</span><span>c</span><!---->"
    );

    mvec.lock_mut().set(1, "B");
    tick().await;
    assert_eq!(
        elem.inner_html(),
        "<span>a</span><span>B</span><span>c</span><!---->"
    );

    mvec.lock_mut().remove(1);
    tick().await;
    assert_eq!(elem.inner_html(), "<span>a</span><span>c</span><!---->");

    mvec.lock_mut().insert(1, "bb");
    tick().await;
    assert_eq!(
        elem.inner_html(),
        "<span>a</span><span>bb</span><span>c</span><!---->"
    );

    mvec.lock_mut().clear();
    tick().await;
    assert_eq!(elem.inner_html(), "<!---->");
}

#[wasm_bindgen_test]
fn test_view() {
    brass::launch(get_root(), || {
        let there = "there";

        let style = brass::signal::signal::Mutable::new("display: block;");

        view! {
            div(
                id="test_view"
                class="lala"
                style=style.signal()
                onMouseMove=|_e: web_sys::MouseEvent| {

                }
                onClick={|| {

                }}
            ) [
                p [
                    "hello"
                    {there}
                ]
            ]
        }
    });

    let html = elem_by_id("test_view").outer_html();
    assert_eq!(
        html,
        r#"<div id="test_view" class="lala"><p>hellothere</p></div>"#
    );
}

// // Refs.

// struct RefComponent {
//     vref: Ref,
// }

// impl brass::Component for RefComponent {
//     type Properties = ();
//     type Msg = ();

//     fn init(_props: Self::Properties, _ctx: &mut brass::Context<Self::Msg>) -> Self {
//         Self { vref: Ref::new() }
//     }

//     fn update(&mut self, _msg: Self::Msg, _ctxx: &mut brass::Context<Self::Msg>) {}

//     fn render(&self, _ctx: &mut brass::RenderContext<Self>) -> brass::VNode {
//         div().attr(Attr::Id, "refcontainer").build_ref(&self.vref)
//     }

//     fn on_property_change(
//         &mut self,
//         _props: Self::Properties,
//         _ctx: &mut brass::Context<Self::Msg>,
//     ) -> brass::ShouldRender {
//         false
//     }

//     fn on_render(&mut self, _first_render: bool) {
//         let elem = self.vref.get().expect("Could not obtain reference");
//         assert_eq!(elem.id(), "refcontainer");

//         let real_elem = brass::util::document()
//             .get_element_by_id("refcontainer")
//             .unwrap();
//         assert_eq!(elem, real_elem);
//     }
// }

// #[wasm_bindgen_test::wasm_bindgen_test]
// fn test_refs() {
//     tracing_wasm::set_as_global_default();
//     brass::boot::<RefComponent>((), get_root());
// }

// // Callbacks.

// pub struct CallbackComp {
//     count: u64,
//     callback: Callback<u64>,
// }

// impl brass::Component for CallbackComp {
//     type Properties = ();

//     type Msg = u64;

//     fn init(_props: Self::Properties, ctx: &mut brass::Context<Self::Msg>) -> Self {
//         Self {
//             count: 0,
//             callback: ctx.callback(),
//         }
//     }

//     fn update(&mut self, msg: Self::Msg, _ctx: &mut brass::Context<Self::Msg>) {
//         self.count += msg;
//     }

//     fn render(&self, _ctx: &mut brass::RenderContext<Self>) -> brass::VNode {
//         div()
//             .and((
//                 div().class("callback-test").and(self.count.to_string()),
//                 vdom::button()
//                     .class("callback-test-trigger")
//                     .on_callback(|_: ClickEvent| 1, &self.callback),
//             ))
//             .build()
//     }

//     fn on_property_change(
//         &mut self,
//         _props: Self::Properties,
//         _ctx: &mut brass::Context<Self::Msg>,
//     ) -> brass::ShouldRender {
//         true
//     }
// }

// #[wasm_bindgen_test::wasm_bindgen_test]
// fn test_callback() {
//     brass::boot::<CallbackComp>((), get_root());
//     let btn =
//         brass::util::query_selector_as::<web_sys::HtmlElement>(".callback-test-trigger").unwrap();
//     btn.click();
//     btn.click();
// }
