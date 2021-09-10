use brass::{
    dom::{Attr, Event},
    vdom::{button, div, div_with, h1, span_with},
};
use wasm_bindgen::{prelude::*, JsCast};

#[wasm_bindgen(start)]
pub fn run() {
    brass::boot::<Root>(
        (),
        brass::util::query_selector("#app")
            .unwrap()
            .dyn_into()
            .unwrap(),
    );
}

struct Root {
    item_count: u64,
}

enum RootMsg {
    IncrementItemCount(u64),
    Clear,
}

impl brass::Component for Root {
    type Properties = ();

    type Msg = RootMsg;

    fn init(_props: Self::Properties, _ctx: &mut brass::Context<Self::Msg>) -> Self {
        Self { item_count: 0 }
    }

    fn update(&mut self, msg: Self::Msg, _ctx: &mut brass::Context<Self::Msg>) {
        match msg {
            RootMsg::IncrementItemCount(count) => {
                self.item_count += count;
            }
            RootMsg::Clear => {
                self.item_count = 0;
            }
        }
    }

    fn render(&self, ctx: &mut brass::RenderContext<Self>) -> brass::VNode {
        let add_1 = div_with(
            button()
                .attr(Attr::Id, "add-1-element")
                .and("Add 1 item")
                .on(
                    Event::Click,
                    ctx.on_simple(|| RootMsg::IncrementItemCount(1)),
                ),
        );
        let add_1000 = div_with(
            button()
                .attr(Attr::Id, "add-1000-elements")
                .and("Add 1000 items")
                .on(
                    Event::Click,
                    ctx.on_simple(|| RootMsg::IncrementItemCount(1000)),
                ),
        );
        let clear = div_with(
            button()
                .and("Clear")
                .attr(Attr::Id, "clear")
                .on(Event::Click, ctx.on_simple(|| RootMsg::Clear)),
        );
        let controls = div().and((add_1, add_1000, clear));

        let items = (0..self.item_count).map(|index| ItemProps { index });
        let item_wrapper = div().and_iter(items);
        div().and((controls, item_wrapper)).build()
    }

    fn on_property_change(
        &mut self,
        _props: Self::Properties,
        _ctx: &mut brass::Context<Self::Msg>,
    ) -> brass::ShouldRender {
        false
    }
}

struct ItemProps {
    index: u64,
}

struct Item {
    index: u64,
    name: String,
    counter: u64,
}

brass::enable_props!(ItemProps => Item);

enum ItemMsg {
    Increment,
}

impl brass::Component for Item {
    type Properties = ItemProps;

    type Msg = ItemMsg;

    fn init(props: Self::Properties, _ctx: &mut brass::Context<Self::Msg>) -> Self {
        let index = props.index;
        Self {
            index,
            name: format!("Item {}", index),
            counter: index,
        }
    }

    fn update(&mut self, msg: Self::Msg, _ctx: &mut brass::Context<Self::Msg>) {
        match msg {
            ItemMsg::Increment => {
                self.counter += 1;
            }
        }
    }

    fn render(&self, ctx: &mut brass::RenderContext<Self>) -> brass::VNode {
        let name = h1().and(&self.name);
        let counter = div().class("counter").and((
            span_with(&self.counter.to_string()),
            button()
                .on(Event::Click, ctx.on_simple(|| ItemMsg::Increment))
                .and("Increment"),
        ));
        div().class("item").and((name, counter)).build()
    }

    fn on_property_change(
        &mut self,
        props: Self::Properties,
        ctx: &mut brass::Context<Self::Msg>,
    ) -> brass::ShouldRender {
        if props.index != self.index {
            *self = Self::init(props, ctx);
            true
        } else {
            false
        }
    }
}
