use brass::RenderContext;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use brass::{
    dom::{Attr, Event},
    vdom::{button, component, div, div_with, h2, h4, input, li, span_with, text, ul},
    Component, Context, ShouldRender,
};

type TodoId = usize;

#[derive(Debug)]
struct Counter {
    count: u32,
    nest: bool,
}

impl Component for Counter {
    type Properties = bool;

    type Msg = u32;

    fn init(props: Self::Properties, _context: &mut Context<Self::Msg>) -> Self {
        Self {
            count: 0,
            nest: props,
        }
    }

    fn on_property_change(
        &mut self,
        _new_props: Self::Properties,
        _context: &mut Context<Self::Msg>,
    ) -> ShouldRender {
        true
    }

    fn update(&mut self, msg: Self::Msg, _context: &mut Context<Self::Msg>) {
        self.count += msg;
    }

    fn render(&self, ctx: &mut RenderContext<Self>) -> brass::VNode {
        let increment = button()
            .and("+")
            .on(Event::Click, ctx.on(|_ev: web_sys::Event| 1));
        div()
            .and("Counter: ")
            .and(self.count.to_string())
            .and(div_with(increment))
            .and_if(self.nest, || {
                div_with(component::<Counter>(false)).attr(Attr::Style, "padding-left: 2rem;")
            })
            .build()
    }
}

#[derive(Debug)]
struct Todo {
    task: String,
    done: bool,
}

#[derive(Debug)]
struct App {
    new_todo: String,
    todos: Vec<Todo>,
}

#[derive(Debug)]
enum Msg {
    Change(String),
    Add,
    ToggleDone(TodoId),
    // Remove(TodoId),
}

impl brass::Component for App {
    type Properties = ();
    type Msg = Msg;

    fn init(_props: Self::Properties, _ctx: &mut Context<Self::Msg>) -> Self {
        Self {
            new_todo: String::new(),
            todos: Vec::new(),
        }
    }

    fn on_property_change(
        &mut self,
        _new_props: Self::Properties,
        _context: &mut Context<Self::Msg>,
    ) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Msg, _ctx: &mut Context<Self::Msg>) {
        match msg {
            Msg::Change(value) => {
                self.new_todo = value;
            }
            Msg::Add => {
                let trimmed = self.new_todo.trim();
                if !trimmed.is_empty() {
                    self.todos.push(Todo {
                        task: trimmed.to_string(),
                        done: false,
                    });
                    self.new_todo.clear();
                }
            }
            Msg::ToggleDone(id) => {
                self.todos.get_mut(id).map(|x| x.done = !x.done);
            } // Msg::Remove(id) => {
              //     self.todos.remove(id);
              // }
        }
    }

    fn render(&self, ctx: &mut RenderContext<Self>) -> brass::VNode {
        let editor = div().and(text("New Todo2: ")).and(
            input()
                .attr(Attr::Value, &self.new_todo)
                .on(
                    Event::Input,
                    ctx.on(|ev: web_sys::Event| {
                        let elem: web_sys::HtmlInputElement =
                            ev.current_target().unwrap().unchecked_into();
                        let value = elem.value();
                        Msg::Change(value)
                    }),
                )
                .on(
                    Event::KeyDown,
                    ctx.on_opt(|ev: web_sys::Event| {
                        let kev: web_sys::KeyboardEvent = ev.unchecked_into();
                        if kev.code() == "Enter" {
                            Some(Msg::Add)
                        } else {
                            None
                        }
                    }),
                ),
        );

        let mut todos = div().and(h4().and("Your Todos:"));

        if self.todos.is_empty() {
            todos.add_child(text("No todos created yet. Don't be so lazy!"));
        } else {
            let mut ul = ul();

            for (id, todo) in self.todos.iter().enumerate() {
                tracing::trace!(?todo, "rendering todo");

                let checked = if todo.done { "1" } else { "" };
                let checkbox = input()
                    .attr(Attr::Type, "checkbox")
                    .attr(Attr::Checked, checked)
                    .on(
                        Event::Click,
                        ctx.on(move |_ev: web_sys::Event| Msg::ToggleDone(id)),
                    );

                let label = if todo.done {
                    span_with(&todo.task).attr(Attr::Style, "text-decoration: line-through;")
                } else {
                    span_with(&todo.task)
                };

                // let remove = button().and("Remove");

                let n = li().and(checkbox).and(label);
                ul.add_child(n);
            }
            todos.add_child(ul);
        }

        div()
            .and(h2().and(text("Copper - Todo")))
            .and(editor)
            .and(todos)
            .build()

        // let mut buttons = div().class("buttons");

        // if self.counter > 0 {
        //     buttons.add_child(button().child(text("-")).on(Event::Click, |_ev| Some(-1)));
        // }
        // buttons.add_child(button().child(text("+")).on(Event::Click, |_ev| Some(1)));

        // let content = div()
        //     .child(div().child(text("hello")))
        //     .child(div().child(text(self.counter.to_string())))
        //     .child(buttons);

        // content.build()
    }
}

#[wasm_bindgen(start)]
pub fn boot() {
    tracing_wasm::set_as_global_default_with_config(tracing_wasm::WASMLayerConfig {
        report_logs_in_console: true,
        report_logs_in_timings: false,
        use_console_color: false,
    });

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    tracing::info!("tracing initialized");
    let elem = brass::util::query_selector("#app")
        .expect("Could not get app")
        .dyn_into()
        .unwrap();
    brass::boot::<App>((), elem);
}
