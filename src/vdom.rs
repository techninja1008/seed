pub mod alias;
pub use alias::*;

pub mod app;
pub mod builder;
pub mod effects;
pub mod mailbox;
pub mod next_tick;
pub mod render_timestamp_delta;

pub struct UndefinedGMsg;

#[cfg(test)]
pub mod tests {
    use futures::future::{self, Future};
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    use web_sys::{self, Element};

    use crate as seed;
    use crate::{class, patch, prelude::*, util, vdom::mailbox::Mailbox, websys_bridge};

    wasm_bindgen_test_configure!(run_in_browser);

    #[derive(Clone, Debug)]
    enum Msg {}

    struct Model {}

    fn create_app() -> App<Msg, Model, Node<Msg>> {
        App::build(|_,_| Init::new(Model {}), |_, _, _| (), |_| seed::empty())
            // mount to the element that exists even in the default test html
            .mount(util::body())
            .finish()
    }

    fn call_patch(
        doc: &web_sys::Document,
        parent: &Element,
        mailbox: &Mailbox<Msg>,
        old_vdom: Node<Msg>,
        mut new_vdom: Node<Msg>,
        app: &App<Msg, Model, Node<Msg>>,
    ) -> Node<Msg> {
        patch::patch(&doc, old_vdom, &mut new_vdom, parent, None, mailbox, &app);
        new_vdom
    }

    fn iter_nodelist(list: web_sys::NodeList) -> impl Iterator<Item = web_sys::Node> {
        (0..list.length()).map(move |i| list.item(i).unwrap())
    }

    fn iter_child_nodes(node: &web_sys::Node) -> impl Iterator<Item = web_sys::Node> {
        iter_nodelist(node.child_nodes())
    }

    #[wasm_bindgen_test]
    fn el_added() {
        let app = create_app();
        let mailbox = Mailbox::new(|_msg: Msg| {});

        let doc = util::document();
        let parent = doc.create_element("div").unwrap();

        let mut vdom = Node::Element(El::empty(seed::dom_types::Tag::Div));
        websys_bridge::assign_ws_nodes(&doc, &mut vdom);
        // clone so we can keep using it after vdom is modified
        if let Node::Element(vdom_el) = vdom.clone() {
            let old_ws = vdom_el.node_ws.as_ref().unwrap().clone();
            parent.append_child(&old_ws).unwrap();

            assert_eq!(parent.children().length(), 1);
            assert_eq!(old_ws.child_nodes().length(), 0);

            vdom = call_patch(&doc, &parent, &mailbox, vdom, div!["text"], &app);
            assert_eq!(parent.children().length(), 1);
            assert!(old_ws.is_same_node(parent.first_child().as_ref()));
            assert_eq!(old_ws.child_nodes().length(), 1);
            assert_eq!(
                old_ws.first_child().unwrap().text_content().unwrap(),
                "text"
            );

            call_patch(
                &doc,
                &parent,
                &mailbox,
                vdom,
                div!["text", "more text", vec![li!["even more text"]]],
                &app,
            );

            assert_eq!(parent.children().length(), 1);
            assert!(old_ws.is_same_node(parent.first_child().as_ref()));
            assert_eq!(old_ws.child_nodes().length(), 3);
            assert_eq!(
                old_ws
                    .child_nodes()
                    .item(0)
                    .unwrap()
                    .text_content()
                    .unwrap(),
                "text"
            );
            assert_eq!(
                old_ws
                    .child_nodes()
                    .item(1)
                    .unwrap()
                    .text_content()
                    .unwrap(),
                "more text"
            );
            let child3 = old_ws.child_nodes().item(2).unwrap();
            assert_eq!(child3.node_name(), "LI");
            assert_eq!(child3.text_content().unwrap(), "even more text");
        } else {
            panic!("Node not Element")
        }
    }

    #[wasm_bindgen_test]
    fn el_removed() {
        let app = create_app();
        let mailbox = Mailbox::new(|_msg: Msg| {});

        let doc = util::document();
        let parent = doc.create_element("div").unwrap();

        let mut vdom = Node::Element(El::empty(seed::dom_types::Tag::Div));
        websys_bridge::assign_ws_nodes(&doc, &mut vdom);
        // clone so we can keep using it after vdom is modified
        if let Node::Element(vdom_el) = vdom.clone() {
            let old_ws = vdom_el.node_ws.as_ref().unwrap().clone();
            parent.append_child(&old_ws).unwrap();

            // First add some child nodes using the vdom
            vdom = call_patch(
                &doc,
                &parent,
                &mailbox,
                vdom,
                div!["text", "more text", vec![li!["even more text"]]],
                &app,
            );

            assert_eq!(parent.children().length(), 1);
            assert_eq!(old_ws.child_nodes().length(), 3);
            let old_child1 = old_ws.child_nodes().item(0).unwrap();

            // Now test that patch function removes the last 2 nodes
            call_patch(&doc, &parent, &mailbox, vdom, div!["text"], &app);

            assert_eq!(parent.children().length(), 1);
            assert!(old_ws.is_same_node(parent.first_child().as_ref()));
            assert_eq!(old_ws.child_nodes().length(), 1);
            assert!(old_child1.is_same_node(old_ws.child_nodes().item(0).as_ref()));
        }
    }

    #[wasm_bindgen_test]
    fn el_changed() {
        let app = create_app();
        let mailbox = Mailbox::new(|_msg: Msg| {});

        let doc = util::document();
        let parent = doc.create_element("div").unwrap();

        let mut vdom = Node::Element(El::empty(seed::dom_types::Tag::Div));
        websys_bridge::assign_ws_nodes(&doc, &mut vdom);
        // clone so we can keep using it after vdom is modified
        if let Node::Element(el) = vdom.clone() {
            let old_ws = el.node_ws.as_ref().unwrap().clone();
            parent.append_child(&old_ws).unwrap();

            // First add some child nodes using the vdom
            vdom = call_patch(
                &doc,
                &parent,
                &mailbox,
                vdom,
                div![span!["hello"], ", ", span!["world"]],
                &app,
            );

            assert_eq!(parent.child_nodes().length(), 1);
            assert_eq!(old_ws.child_nodes().length(), 3);

            // Now add some attributes
            call_patch(
                &doc,
                &parent,
                &mailbox,
                vdom,
                div![
                    span![class!["first"], "hello"],
                    ", ",
                    span![class!["second"], "world"],
                ],
                &app,
            );

            let child1 = old_ws
                .child_nodes()
                .item(0)
                .unwrap()
                .dyn_into::<Element>()
                .unwrap();
            assert_eq!(child1.get_attribute("class"), Some("first".to_string()));
            let child3 = old_ws
                .child_nodes()
                .item(2)
                .unwrap()
                .dyn_into::<Element>()
                .unwrap();
            assert_eq!(child3.get_attribute("class"), Some("second".to_string()));
        } else {
            panic!("Node not Element")
        }
    }

    #[wasm_bindgen_test]
    fn els_changed_correct_order() {
        let app = create_app();
        let mailbox = Mailbox::new(|_msg: Msg| {});

        let doc = util::document();
        let parent = doc.create_element("div").unwrap();

        let mut vdom = div![];
        websys_bridge::assign_ws_nodes(&doc, &mut vdom);
        // clone so we can keep using it after vdom is modified
        if let Node::Element(el) = vdom.clone() {
            let old_ws = el.node_ws.as_ref().unwrap().clone();
            parent.append_child(&old_ws).unwrap();

            vdom = call_patch(&doc, &parent, &mailbox, vdom, div!["1", a!["2"]], &app);
            let html_result = old_ws.clone().dyn_into::<Element>().unwrap().inner_html();
            assert_eq!(html_result, "1<a>2</a>");

            call_patch(&doc, &parent, &mailbox, vdom, div![a!["A"], "B"], &app);
            let html_result = old_ws.dyn_into::<Element>().unwrap().inner_html();
            assert_eq!(html_result, "<a>A</a>B");
        } else {
            panic!("Node not Element")
        }
    }

    /// Test if attribute `disabled` is correctly added and then removed.
    #[wasm_bindgen_test]
    fn attr_disabled() {
        let app = create_app();
        let mailbox = Mailbox::new(|_msg: Msg| {});

        let doc = util::document();
        let parent = doc.create_element("div").unwrap();

        let mut vdom = Node::Element(El::empty(seed::dom_types::Tag::Div));
        websys_bridge::assign_ws_nodes(&doc, &mut vdom);
        // clone so we can keep using it after vdom is modified
        if let Node::Element(vdom_el) = vdom.clone() {
            let old_ws = vdom_el.node_ws.as_ref().unwrap().clone();
            parent.append_child(&old_ws).unwrap();

            // First add button without attribute `disabled`
            vdom = call_patch(
                &doc,
                &parent,
                &mailbox,
                vdom,
                div![button![attrs! { At::Disabled => false.as_at_value() }]],
                &app,
            );

            assert_eq!(parent.child_nodes().length(), 1);
            assert_eq!(old_ws.child_nodes().length(), 1);
            let button = old_ws
                .child_nodes()
                .item(0)
                .unwrap()
                .dyn_into::<Element>()
                .unwrap();
            assert_eq!(button.has_attribute("disabled"), false);

            // Now add attribute `disabled`
            vdom = call_patch(
                &doc,
                &parent,
                &mailbox,
                vdom,
                div![button![attrs! { At::Disabled => true.as_at_value() }]],
                &app,
            );

            let button = old_ws
                .child_nodes()
                .item(0)
                .unwrap()
                .dyn_into::<Element>()
                .unwrap();
            assert_eq!(
                button
                    .get_attribute("disabled")
                    .expect("button hasn't got attribute `disabled`!"),
                ""
            );

            // And remove attribute `disabled`
            call_patch(
                &doc,
                &parent,
                &mailbox,
                vdom,
                div![button![attrs! { At::Disabled => false.as_at_value() }]],
                &app,
            );

            let button = old_ws
                .child_nodes()
                .item(0)
                .unwrap()
                .dyn_into::<Element>()
                .unwrap();
            assert_eq!(button.has_attribute("disabled"), false);
        } else {
            panic!("Node not El")
        }
    }

    /// Test that if the first child was a seed::empty() and it is changed to a non-empty El,
    /// then the new element is inserted at the correct position.
    #[wasm_bindgen_test]
    fn empty_changed_in_front() {
        let app = create_app();
        let mailbox = Mailbox::new(|_msg: Msg| {});

        let doc = util::document();
        let parent = doc.create_element("div").unwrap();

        let mut vdom = Node::Element(El::empty(seed::dom_types::Tag::Div));
        websys_bridge::assign_ws_nodes(&doc, &mut vdom);
        // clone so we can keep using it after vdom is modified
        if let Node::Element(vdom_el) = vdom.clone() {
            let old_ws = vdom_el.node_ws.as_ref().unwrap().clone();
            parent.append_child(&old_ws).unwrap();

            assert_eq!(parent.children().length(), 1);
            assert_eq!(old_ws.child_nodes().length(), 0);

            vdom = call_patch(
                &doc,
                &parent,
                &mailbox,
                vdom,
                div![seed::empty(), "b", "c"],
                &app,
            );
            assert_eq!(parent.children().length(), 1);
            assert!(old_ws.is_same_node(parent.first_child().as_ref()));
            assert_eq!(
                iter_child_nodes(&old_ws)
                    .map(|node| node.text_content().unwrap())
                    .collect::<Vec<_>>(),
                &["b", "c"],
            );

            call_patch(&doc, &parent, &mailbox, vdom, div!["a", "b", "c"], &app);

            assert_eq!(parent.children().length(), 1);
            assert!(old_ws.is_same_node(parent.first_child().as_ref()));
            assert_eq!(
                iter_child_nodes(&old_ws)
                    .map(|node| node.text_content().unwrap())
                    .collect::<Vec<_>>(),
                &["a", "b", "c"],
            );
        } else {
            panic!("Not Element node")
        }
    }

    /// Test that if a middle child was a seed::empty() and it is changed to a non-empty El,
    /// then the new element is inserted at the correct position.
    #[wasm_bindgen_test]
    fn empty_changed_in_the_middle() {
        let app = create_app();
        let mailbox = Mailbox::new(|_msg: Msg| {});

        let doc = util::document();
        let parent = doc.create_element("div").unwrap();

        let mut vdom = Node::Element(El::empty(seed::dom_types::Tag::Div));
        websys_bridge::assign_ws_nodes(&doc, &mut vdom);
        if let Node::Element(vdom_el) = vdom.clone() {
            // clone so we can keep using it after vdom is modified
            let old_ws = vdom_el.node_ws.as_ref().unwrap().clone();
            parent.append_child(&old_ws).unwrap();

            assert_eq!(parent.children().length(), 1);
            assert_eq!(old_ws.child_nodes().length(), 0);

            vdom = call_patch(
                &doc,
                &parent,
                &mailbox,
                vdom,
                div!["a", seed::empty(), "c"],
                &app,
            );
            assert_eq!(parent.children().length(), 1);
            assert!(old_ws.is_same_node(parent.first_child().as_ref()));
            assert_eq!(
                iter_child_nodes(&old_ws)
                    .map(|node| node.text_content().unwrap())
                    .collect::<Vec<_>>(),
                &["a", "c"],
            );

            call_patch(&doc, &parent, &mailbox, vdom, div!["a", "b", "c"], &app);

            assert_eq!(parent.children().length(), 1);
            assert!(old_ws.is_same_node(parent.first_child().as_ref()));
            assert_eq!(
                iter_child_nodes(&old_ws)
                    .map(|node| node.text_content().unwrap())
                    .collect::<Vec<_>>(),
                &["a", "b", "c"],
            );
        } else {
            panic!("Not Element node")
        }
    }

    /// Test that if the old_el passed to patch was itself an empty, it is correctly patched to a non-empty.
    #[wasm_bindgen_test]
    fn root_empty_changed() {
        let app = create_app();
        let mailbox = Mailbox::new(|_msg: Msg| {});

        let doc = util::document();
        let parent = doc.create_element("div").unwrap();

        let mut vdom = seed::empty();

        vdom = call_patch(
            &doc,
            &parent,
            &mailbox,
            vdom,
            div!["a", seed::empty(), "c"],
            &app,
        );
        assert_eq!(parent.children().length(), 1);
        if let Node::Element(vdom_el) = vdom {
            let el_ws = vdom_el.node_ws.as_ref().expect("el_ws missing");
            assert!(el_ws.is_same_node(parent.first_child().as_ref()));
            assert_eq!(
                iter_child_nodes(&el_ws)
                    .map(|node| node.text_content().unwrap())
                    .collect::<Vec<_>>(),
                &["a", "c"],
            );
        } else {
            panic!("Node not Element type")
        }
    }

    /// Test that an empty->empty transition is handled correctly.
    #[wasm_bindgen_test]
    fn root_empty_to_empty() {
        let app = create_app();
        let mailbox = Mailbox::new(|_msg: Msg| {});

        let doc = util::document();
        let parent = doc.create_element("div").unwrap();

        let old = seed::empty();
        call_patch(&doc, &parent, &mailbox, old, seed::empty(), &app);
        assert_eq!(parent.children().length(), 0);
    }

    /// Test that a text Node is correctly patched to an Element and vice versa.
    #[wasm_bindgen_test]
    fn text_to_element_to_text() {
        let app = create_app();
        let mailbox = Mailbox::new(|_msg: Msg| {});

        let doc = util::document();
        let parent = doc.create_element("div").unwrap();

        let mut vdom = seed::empty();
        vdom = call_patch(&doc, &parent, &mailbox, vdom, Node::new_text("abc"), &app);
        assert_eq!(parent.child_nodes().length(), 1);
        let text = parent
            .first_child()
            .unwrap()
            .dyn_ref::<web_sys::Text>()
            .expect("not a Text node")
            .clone();
        assert_eq!(text.text_content().unwrap(), "abc");

        // change to a span (that contains a text node and styling).
        // span was specifically chosen here because text Els are saved with the span tag.
        // (or at least they were when the test was written.)
        vdom = call_patch(
            &doc,
            &parent,
            &mailbox,
            vdom,
            span![style!["color" => "red"], "def"],
            &app,
        );
        assert_eq!(parent.child_nodes().length(), 1);
        let element = parent
            .first_child()
            .unwrap()
            .dyn_ref::<Element>()
            .expect("not an Element node")
            .clone();
        assert_eq!(&element.tag_name().to_lowercase(), "span");

        // change back to a text node
        call_patch(&doc, &parent, &mailbox, vdom, Node::new_text("abc"), &app);
        assert_eq!(parent.child_nodes().length(), 1);
        let text = parent
            .first_child()
            .unwrap()
            .dyn_ref::<web_sys::Text>()
            .expect("not a Text node")
            .clone();
        assert_eq!(text.text_content().unwrap(), "abc");
    }

    /// Test that the lifecycle hooks are called correctly.
    #[wasm_bindgen_test]
    fn lifecycle_hooks() {
        let app = create_app();
        use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

        let mailbox = Mailbox::new(|_msg: Msg| {});

        let doc = util::document();
        let parent = doc.create_element("div").unwrap();

        let mut vdom = seed::empty();

        let node_ref: Rc<RefCell<Option<web_sys::Node>>> = Default::default();
        let mount_op_counter: Rc<AtomicUsize> = Default::default();
        let update_counter: Rc<AtomicUsize> = Default::default();

        // A real view() function would recreate these closures on each call.
        // We create the closures once and then clone them, which is hopefully close enough.
        let did_mount_func = {
            let node_ref = node_ref.clone();
            let mount_op_counter = mount_op_counter.clone();
            move |node: &web_sys::Node| {
                node_ref.borrow_mut().replace(node.clone());
                assert_eq!(
                    mount_op_counter.fetch_add(1, SeqCst),
                    0,
                    "did_mount was called more than once"
                );
            }
        };
        let did_update_func = {
            let update_counter = update_counter.clone();
            move |_node: &web_sys::Node| {
                update_counter.fetch_add(1, SeqCst);
            }
        };
        let will_unmount_func = {
            let node_ref = node_ref.clone();
            move |_node: &web_sys::Node| {
                node_ref.borrow_mut().take();
                // If the counter wasn't 1, then either:
                // * did_mount wasn't called - we already check this elsewhere
                // * did_mount was called more than once - we already check this elsewhere
                // * will_unmount was called more than once
                assert_eq!(
                    mount_op_counter.fetch_add(1, SeqCst),
                    1,
                    "will_unmount was called more than once"
                );
            }
        };

        vdom = call_patch(
            &doc,
            &parent,
            &mailbox,
            vdom,
            div![
                "a",
                did_mount(did_mount_func.clone()),
                did_update(did_update_func.clone()),
                will_unmount(will_unmount_func.clone()),
            ],
            &app,
        );
        assert!(
            node_ref.borrow().is_some(),
            "did_mount wasn't called and should have been"
        );
        assert_eq!(
            update_counter.load(SeqCst),
            0,
            "did_update was called and shouldn't have been"
        );
        let first_child = parent.first_child().unwrap();
        assert!(node_ref
            .borrow()
            .as_ref()
            .unwrap()
            .is_same_node(Some(&first_child)));

        // now modify the element, see if did_update gets called.
        vdom = call_patch(
            &doc,
            &parent,
            &mailbox,
            vdom,
            div![
                "a",
                attrs! {At::Href => "#"},
                did_mount(did_mount_func.clone()),
                did_update(did_update_func.clone()),
                will_unmount(will_unmount_func.clone()),
            ],
            &app,
        );
        assert!(
            node_ref
                .borrow()
                .as_ref()
                .expect("will_unmount was called early")
                .is_same_node(Some(&first_child)),
            "node reference changed"
        );
        assert_eq!(
            update_counter.load(SeqCst),
            1,
            "did_update wasn't called and should have been"
        );

        // and now unmount the element to see if will_unmount gets called.
        call_patch(&doc, &parent, &mailbox, vdom, seed::empty(), &app);
        assert!(node_ref.borrow().is_none(), "will_unmount wasn't called");
    }

    /// Tests an update() function that repeatedly sends messages or performs commands.
    #[wasm_bindgen_test(async)]
    fn update_promises() -> impl Future<Item = (), Error = JsValue> {
        // ARRANGE

        // when we call `test_value_sender.send(..)`, future `test_value_receiver` will be marked as resolved
        let (test_value_sender, test_value_receiver) = futures::oneshot::<Counters>();

        // big numbers because we want to test if it doesn't blow call-stack
        // Note: Firefox has bigger call stack then Chrome - see http://2ality.com/2014/04/call-stack-size.html
        const MESSAGES_TO_SEND: i32 = 5_000;
        const COMMANDS_TO_PERFORM: i32 = 4_000;

        #[derive(Default, Copy, Clone, Debug)]
        struct Counters {
            messages_sent: i32,
            commands_scheduled: i32,
            messages_received: i32,
            commands_performed: i32,
        }

        #[derive(Default)]
        struct Model {
            counters: Counters,
            test_value_sender: Option<futures::sync::oneshot::Sender<Counters>>,
        }
        #[derive(Clone)]
        enum Msg {
            MessageReceived,
            CommandPerformed,
            Start,
        }

        fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
            orders.skip();

            match msg {
                Msg::MessageReceived => model.counters.messages_received += 1,
                Msg::CommandPerformed => model.counters.commands_performed += 1,
                Msg::Start => (),
            }

            if model.counters.messages_sent < MESSAGES_TO_SEND {
                orders.send_msg(Msg::MessageReceived);
                model.counters.messages_sent += 1;
            }
            if model.counters.commands_scheduled < MESSAGES_TO_SEND {
                orders.perform_cmd(future::ok(Msg::CommandPerformed));
                model.counters.commands_scheduled += 1;
            }

            if model.counters.messages_received == MESSAGES_TO_SEND
                && model.counters.commands_performed == COMMANDS_TO_PERFORM
            {
                model
                    .test_value_sender
                    .take()
                    .unwrap()
                    .send(model.counters)
                    .unwrap()
            }
        }

        let app = App::build(
            |_, _| {
                Init::new(Model {
                    test_value_sender: Some(test_value_sender),
                    ..Default::default()
                })
            },
            update,
            |_| seed::empty(),
        )
        .mount(seed::body())
        .finish()
        .run();

        // ACT
        app.update(Msg::Start);

        // ASSERT
        test_value_receiver
            .map(|counters| {
                assert_eq!(counters.messages_received, MESSAGES_TO_SEND);
                assert_eq!(counters.commands_performed, COMMANDS_TO_PERFORM);
            })
            .map_err(|_| panic!("test_value_sender.send probably wasn't called!"))
    }
}
