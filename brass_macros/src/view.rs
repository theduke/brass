use proc_macro::TokenStream;
use quote::quote;

pub fn view(tokens: TokenStream) -> TokenStream {
    let template = syn::parse_macro_input!(tokens as Template);
    let output = render_template(template);
    output.to_string();
    output.into()
}

#[derive(Debug)]
enum Node {
    Elem {
        tag: syn::Ident,
        attributes: Vec<Attr>,
        event_handlers: Vec<EventHandler>,
        children: Vec<Self>,
    },
    Fragment {
        items: Vec<Self>,
    },
    Text {
        value: syn::LitStr,
    },
    Expr {
        expr: syn::Expr,
    },
}

#[derive(Debug)]
struct EventHandler {
    event: syn::Ident,
    handler: syn::Expr,
}

#[derive(Debug)]
struct Attr {
    name: syn::Ident,
    value: AttrValue,
}

#[derive(Debug)]
enum AttrValue {
    None,
    Str(syn::LitStr),
    Expr(syn::Expr),
}

struct Template {
    node: Node,
}

fn render_node(node: Node, nested: bool) -> proc_macro2::TokenStream {
    // TODO: use HTML templates and node.cloneNode(true) for better performance.
    // (similar to solidjs)

    match node {
        Node::Elem {
            tag,
            attributes,
            event_handlers,
            children,
        } => {
            let attrs = attributes.into_iter().map(|attr| {
                let mut name_value = attr.name.to_string();
                name_value.replace_range(0..1, &name_value[0..1].to_uppercase());
                let name = proc_macro2::Ident::new(&name_value, attr.name.span());
                match attr.value {
                AttrValue::None => {
                    quote! {
                        parent.add_attr(brass::dom::Attr::#name, brass::web::empty_string());
                    }
                }
                AttrValue::Str(value) => {
                    // TODO: use cached JsStr / wasm_bindgen constant
                    quote!{
                        parent.add_attr(brass::dom::Attr::#name, #value);
                    }
                }
                AttrValue::Expr(e) => {
                    quote!{
                        brass::dom::AttrValueApply::attr_apply(#e, brass::dom::Attr::#name, &mut parent);
                    }
                }
            }
            });

            let event_handlers = event_handlers.into_iter().map(|e| {

                let name = &e.event.to_string()[2..];
                let ev_ident = syn::Ident::new(name, e.event.span());
                let handler = e.handler;

                quote!{
                    brass::dom::EventHandlerApply::event_handler_apply(#handler, brass::dom::Event::#ev_ident, &mut parent);
                    // parent.add_event_listener_cast(
                    //     brass::dom::Event::#ev_ident,
                    //     #handler
                    // );
                }

            });

            let children = children.into_iter().map(|c| render_node(c, true));

            let mut tag_name = tag.to_string();
            tag_name.replace_range(0..1, &tag_name[0..1].to_uppercase());
            let tag_ident = syn::Ident::new(&tag_name, tag.span());

            let builder = quote! {
                {
                    let mut parent = brass::dom::TagBuilder::new(brass::dom::Tag::#tag_ident);
                    #(#attrs)*
                    #(#event_handlers)*
                    #(#children)*
                    parent
                }
            };

            if nested {
                quote! {
                    parent.add_child(#builder);
                }
            } else {
                quote! {
                    brass::dom::View::Node(#builder.build())
                }
            }
        }
        Node::Fragment { items } => {
            if nested {
                let items = items.into_iter().map(|item| render_node(item, true));
                quote! {
                    #(#items)*
                }
            } else {
                let items = items.into_iter().map(|item| render_node(item, false));
                quote! {
                    {
                        let mut f = brass::dom::Fragment{ items: Vec::new() };
                        #(
                            f.items.push(#items);
                        )*
                        brass::dom::View::Fragment(f)
                    }
                }
            }
        }
        Node::Text { value } => {
            if nested {
                quote! {
                    parent.add_text(brass::web::DomStr::Str(#value));
                }
            } else {
                // TODO: use cached JsString ?
                quote! {
                    brass::dom::View::Node(
                        brass::dom::View::Node(brass::dom::Node::new_text(brass::web::DomStr::Str(#value)))
                    )
                }
            }
        }
        Node::Expr { expr } => {
            if nested {
                quote! {
                    brass::dom::Apply::apply(#expr, &mut parent);
                }
            } else {
                todo!()
            }
        }
    }
}

fn render_template(tpl: Template) -> proc_macro2::TokenStream {
    render_node(tpl.node, false)
}

impl syn::parse::Parse for Template {
    fn parse(stream: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items = Vec::<Node>::new();

        while !stream.is_empty() {
            items.push(stream.parse()?);
        }

        let node = if items.len() == 1 {
            items.pop().unwrap()
        } else {
            Node::Fragment { items }
        };

        Ok(Self { node })
    }
}

impl syn::parse::Parse for Node {
    fn parse(stream: syn::parse::ParseStream) -> syn::Result<Self> {
        if stream.peek(syn::LitStr) {
            Ok(Self::Text {
                value: stream.parse()?,
            })
        } else if stream.peek(syn::Ident) {
            let tag = stream.parse()?;

            let mut attributes = Vec::<Attr>::new();
            let mut event_handlers = Vec::<EventHandler>::new();

            if stream.peek(syn::token::Paren) {
                let inner;
                syn::parenthesized!(inner in stream);

                while !inner.is_empty() {
                    let name: syn::Ident = inner.parse()?;

                    let name_value = name.to_string();
                    if name_value.starts_with("on") {
                        inner.parse::<syn::token::Eq>()?;
                        let handler: syn::Expr = inner.parse()?;

                        event_handlers.push(EventHandler {
                            event: name,
                            handler,
                        });

                        continue;
                    }

                    let value = if inner.peek(syn::token::Eq) {
                        inner.parse::<syn::token::Eq>()?;

                        let expr: syn::Expr = inner.parse()?;

                        match expr {
                            syn::Expr::Lit(lit) => match lit.lit {
                                syn::Lit::Str(s) => AttrValue::Str(s),
                                other => {
                                    return Err(syn::parse::Error::new_spanned(
                                        other,
                                        "Invalid attribute value",
                                    ));
                                }
                            },
                            other => {
                                AttrValue::Expr(other)
                                // return Err(syn::parse::Error::new_spanned(
                                //     other,
                                //     "Unexpected input",
                                // ));
                            }
                        }
                    } else {
                        AttrValue::None
                    };

                    attributes.push(Attr { name, value });

                    // Skip optional trailing comma.
                    if inner.peek(syn::token::Comma) {
                        inner.parse::<syn::token::Comma>()?;
                    }
                }
            }

            let mut children = Vec::new();
            if stream.peek(syn::token::Bracket) {
                let inner;
                syn::bracketed!(inner in stream);

                while !inner.is_empty() {
                    let child: Node = inner.parse()?;
                    children.push(child);
                }
            }

            Ok(Self::Elem {
                tag,
                attributes,
                event_handlers,
                children,
            })
        } else if stream.peek(syn::token::Brace) {
            let inner;
            syn::braced!(inner in stream);
            let expr: syn::Expr = inner.parse()?;
            Ok(Self::Expr { expr })
        } else {
            Err(stream.error("Unexpected input"))
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::*;

    #[test]
    fn test_parse_node_text() {
        let input = quote! {
            "hello"
        };
        let node: Node = syn::parse2(input).unwrap();
        match node {
            Node::Text { value } => {
                assert_eq!(value.value(), "hello");
            }
            other => {
                panic!("Expected text, got {:?}", other)
            }
        }
    }

    #[test]
    fn test_parse_node_elem_plain() {
        let input = quote! {
            div
        };
        let node: Node = syn::parse2(input).unwrap();
        match node {
            Node::Elem { tag, .. } => {
                assert_eq!(tag.to_string(), "div");
            }

            other => {
                panic!("Expected text, got {:?}", other)
            }
        }
    }

    #[test]
    fn test_parse_node_elem_with_attrs_lit() {
        let input = quote! {
            div(style="lala" height=22/100 width="200" )
        };
        let node: Node = syn::parse2(input).unwrap();
        match node {
            Node::Elem {
                tag, attributes, ..
            } => {
                assert_eq!(tag.to_string(), "div");

                assert_eq!(attributes[0].name.to_string(), "style");
                match &attributes[0].value {
                    AttrValue::Str(s) => {
                        assert_eq!(s.value(), "lala");
                    }
                    other => {
                        panic!("Invalid attribute value: {:?}", other);
                    }
                }

                assert_eq!(attributes[1].name.to_string(), "height");
                match &attributes[1].value {
                    AttrValue::Expr(_) => {}
                    other => {
                        panic!("Invalid attribute value: {:?}", other);
                    }
                }

                assert_eq!(attributes[2].name.to_string(), "width");
                match &attributes[2].value {
                    AttrValue::Str(s) => {
                        assert_eq!(s.value(), "200");
                    }
                    other => {
                        panic!("Invalid attribute value: {:?}", other);
                    }
                }
            }

            other => {
                panic!("Expected text, got {:?}", other)
            }
        }
    }

    #[test]
    fn test_parse_node_with_empty_children() {
        let input = quote! {
            div {}
        };
        let node: Node = syn::parse2(input).unwrap();
        match node {
            Node::Elem { tag, children, .. } => {
                assert_eq!(tag.to_string(), "div");
                assert!(children.is_empty());
            }
            other => {
                panic!("Expected elem, got {:?}", other)
            }
        }
    }

    #[test]
    fn test_parse_node_with_children() {
        let input = quote! {
            div {
                "hello"
                p { "no" }
            }
        };
        let node: Node = syn::parse2(input).unwrap();
        match node {
            Node::Elem { tag, children, .. } => {
                assert_eq!(tag.to_string(), "div");
                assert_eq!(children.len(), 2);
            }
            other => {
                panic!("Expected elem, got {:?}", other)
            }
        }
    }
}
