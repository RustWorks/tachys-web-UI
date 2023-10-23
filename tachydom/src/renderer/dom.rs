use super::{CastFrom, DomRenderer, Renderer};
use crate::{dom::document, ok_or_debug, or_debug, view::Mountable};
//use std::cell::RefCell;
use wasm_bindgen::{intern, JsCast, JsValue};
use web_sys::{
    Comment, CssStyleDeclaration, DocumentFragment, DomTokenList, Element,
    HtmlElement, Node, Text,
};

pub struct Dom;

/* #[derive(Debug, Default)]
struct PendingStuff {
    pub text: String,
    pub nodes: Vec<u32>,
    pub lengths: Vec<u32>,
}

thread_local! {
    static PENDING: RefCell<PendingStuff> = Default::default();
} */

/* #[wasm_bindgen(inline_js = "
let __NODE_HEAP = [];
export function flush_nodes(bytes, lengths, nodes) {
        let utf8decoder = new TextDecoder();
        let all_text = utf8decoder.decode(bytes);
        let last = 0;
        for(let i = 0; i < lengths.length; i++) {
            let length = lengths[i];
            let text = all_text.slice(last, last + length);
            last = last + length;
            __NODE_HEAP[nodes[i]].nodeValue = text;
        }
        __NODE_HEAP = [];
    }

    export function add_node_to_heap(node) {
        __NODE_HEAP.push(node);
        return __NODE_HEAP.length - 1;
    }")]
extern "C" {
    fn flush_nodes(bytes: &[u8], lengths: &[u32], nodes: &[u32]);

    fn add_node_to_heap(node: &Text) -> u32;
}

impl Dom {
    pub fn flush() {
        PENDING.with(|t| {
            let mut t = t.borrow_mut();
            flush_nodes(
                std::mem::take(&mut t.text).as_bytes(),
                &std::mem::take(&mut t.lengths),
                &std::mem::take(&mut t.nodes),
            );
        })
    }
} */

impl Renderer for Dom {
    type Node = Node;
    type Text = Text;
    type Element = Element;
    type Placeholder = Comment;

    fn create_text_node(text: &str) -> Self::Text {
        document().create_text_node(text)
    }

    fn create_placeholder() -> Self::Placeholder {
        document().create_comment("")
    }

    fn set_text(node: &Self::Text, text: &str) {
        /* PENDING.with(|p| {
            let mut p = p.borrow_mut();
            p.text.push_str(text);
            let idx = add_node_to_heap(node);
            p.nodes.push(idx);
            p.lengths.push(text.len() as u32);
        }); */
        node.set_node_value(Some(text));
    }

    fn set_attribute(node: &Self::Element, name: &str, value: &str) {
        or_debug!(
            node.set_attribute(intern(name), value),
            node,
            "setAttribute"
        );
    }

    fn remove_attribute(node: &Self::Element, name: &str) {
        or_debug!(node.remove_attribute(intern(name)), node, "removeAttribute");
    }

    fn insert_node(
        parent: &Self::Element,
        new_child: &Self::Node,
        anchor: Option<&Self::Node>,
    ) {
        ok_or_debug!(
            parent.insert_before(new_child, anchor),
            parent,
            "insertNode"
        );
    }

    fn remove_node(
        parent: &Self::Element,
        child: &Self::Node,
    ) -> Option<Self::Node> {
        ok_or_debug!(parent.remove_child(child), parent, "removeNode")
    }

    fn remove(node: &Self::Node) {
        node.unchecked_ref::<Element>().remove();
    }

    fn get_parent(node: &Self::Node) -> Option<Self::Node> {
        node.parent_node()
    }

    fn first_child(node: &Self::Node) -> Option<Self::Node> {
        node.first_child()
    }

    fn next_sibling(node: &Self::Node) -> Option<Self::Node> {
        node.next_sibling()
    }

    fn log_node(node: &Self::Node) {
        web_sys::console::log_1(node);
    }

    fn clear_children(parent: &Self::Element) {
        parent.set_text_content(Some(""));
    }
}

impl DomRenderer for Dom {
    type Event = JsValue;
    type ClassList = DomTokenList;
    type CssStyleDeclaration = CssStyleDeclaration;

    fn set_property(el: &Self::Element, key: &str, value: &JsValue) {
        or_debug!(
            js_sys::Reflect::set(
                el,
                &wasm_bindgen::JsValue::from_str(intern(key)),
                &value,
            ),
            el,
            "setProperty"
        );
    }

    fn add_event_listener(
        el: &Self::Element,
        name: &str,
        cb: Box<dyn FnMut(Self::Event)>,
    ) {
        let cb = wasm_bindgen::closure::Closure::wrap(cb).into_js_value();
        or_debug!(
            el.add_event_listener_with_callback(
                intern(name),
                cb.as_ref().unchecked_ref()
            ),
            el,
            "addEventListener"
        );
    }

    fn class_list(el: &Self::Element) -> Self::ClassList {
        el.class_list()
    }

    fn add_class(list: &Self::ClassList, name: &str) {
        or_debug!(list.add_1(intern(name)), list.unchecked_ref(), "add()");
    }

    fn remove_class(list: &Self::ClassList, name: &str) {
        or_debug!(
            list.remove_1(intern(name)),
            list.unchecked_ref(),
            "remove()"
        );
    }

    fn style(el: &Self::Element) -> Self::CssStyleDeclaration {
        el.unchecked_ref::<HtmlElement>().style()
    }

    fn set_css_property(
        style: &Self::CssStyleDeclaration,
        name: &str,
        value: &str,
    ) {
        or_debug!(
            style.set_property(intern(name), value),
            style.unchecked_ref(),
            "setProperty"
        );
    }
}

impl Mountable<Dom> for Node {
    fn unmount(&mut self) {
        todo!()
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        Dom::insert_node(parent, self, marker);
    }

    fn insert_before_this(
        &self,
        parent: &<Dom as Renderer>::Element,
        child: &mut dyn Mountable<Dom>,
    ) -> bool {
        child.mount(parent, Some(self));
        true
    }
}

impl Mountable<Dom> for Text {
    fn unmount(&mut self) {
        todo!()
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        Dom::insert_node(parent, self, marker);
    }

    fn insert_before_this(
        &self,
        parent: &<Dom as Renderer>::Element,
        child: &mut dyn Mountable<Dom>,
    ) -> bool {
        child.mount(parent, Some(self.as_ref()));
        true
    }
}

impl Mountable<Dom> for Comment {
    fn unmount(&mut self) {
        todo!()
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        Dom::insert_node(parent, self, marker);
    }

    fn insert_before_this(
        &self,
        parent: &<Dom as Renderer>::Element,
        child: &mut dyn Mountable<Dom>,
    ) -> bool {
        child.mount(parent, Some(self.as_ref()));
        true
    }
}

impl Mountable<Dom> for Element {
    fn unmount(&mut self) {
        todo!()
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        Dom::insert_node(parent, self, marker);
    }

    fn insert_before_this(
        &self,
        parent: &<Dom as Renderer>::Element,
        child: &mut dyn Mountable<Dom>,
    ) -> bool {
        child.mount(parent, Some(self.as_ref()));
        true
    }
}

impl Mountable<Dom> for DocumentFragment {
    fn unmount(&mut self) {
        todo!()
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        Dom::insert_node(parent, self, marker);
    }

    fn insert_before_this(
        &self,
        parent: &<Dom as Renderer>::Element,
        child: &mut dyn Mountable<Dom>,
    ) -> bool {
        child.mount(parent, Some(self.as_ref()));
        true
    }
}

impl CastFrom<Node> for Text {
    fn cast_from(node: Node) -> Option<Text> {
        node.clone().dyn_into().ok()
    }
}

impl CastFrom<Node> for Comment {
    fn cast_from(node: Node) -> Option<Comment> {
        node.clone().dyn_into().ok()
    }
}

impl CastFrom<Node> for Element {
    fn cast_from(node: Node) -> Option<Element> {
        node.clone().dyn_into().ok()
    }
}
