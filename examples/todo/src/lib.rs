use brass::{
    dom::{ClickEvent, Tag, TagBuilder},
    effect::set_interval,
    signal::{
        signal::{Mutable, SignalExt},
        signal_vec::MutableVec,
    },
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    tracing_wasm::set_as_global_default_with_config(tracing_wasm::WASMLayerConfig {
        report_logs_in_console: true,
        report_logs_in_timings: false,
        use_console_color: false,
    });
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = web_sys::window().unwrap();
    let doc = window.document().unwrap();
    let root = doc.create_element("div").unwrap();
    doc.body().unwrap().append_child(&root).unwrap();

    brass::launch(root, || {
        let counter = Mutable::new(0u64);
        let items = MutableVec::new_with_values(vec!["a".to_string()]);
        let items_signal = items.signal_vec_cloned();

        let mut index = 0;
        let interval = set_interval(std::time::Duration::from_secs(1), move || {
            tracing::trace!("tick");
            index += 1;
            items.lock_mut().push_cloned(index.to_string());
        });

        let counter2 = counter.clone();
        TagBuilder::new(Tag::Div)
            .bind(interval)
            .child(
                TagBuilder::new(Tag::Button)
                    .and("+")
                    .text_signal(counter.signal().map(|x| x.to_string()))
                    .on(move |_: ClickEvent| {
                        tracing::trace!("event handler!");
                        counter2.replace_with(|v| *v + 1);
                    }),
            )
            .signal_vec(items_signal, |value| {
                TagBuilder::new(Tag::Div).text(value.as_str()).build()
            })
            .into_view()
    });
}
