//! A stupidly-simple mock DOM implementation that can be used for testing.
//!
//! Do not use this for anything real.

use super::{CastFrom, Renderer};
use crate::{html::element::ElementType, view::Mountable};
use slotmap::{new_key_type, SlotMap};
use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};

pub struct MockDom;

new_key_type! {
    struct NodeId;
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Node(NodeId);

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Element(Node);

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Text(Node);

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Fragment(Node);

impl AsRef<Node> for Node {
    fn as_ref(&self) -> &Node {
        self
    }
}

impl AsRef<Node> for Element {
    fn as_ref(&self) -> &Node {
        &self.0
    }
}

impl AsRef<Node> for Text {
    fn as_ref(&self) -> &Node {
        &self.0
    }
}

impl AsRef<Node> for Fragment {
    fn as_ref(&self) -> &Node {
        &self.0
    }
}

pub fn node_eq(a: impl AsRef<Node>, b: impl AsRef<Node>) -> bool {
    a.as_ref() == b.as_ref()
}

impl From<Text> for Node {
    fn from(value: Text) -> Self {
        Node(value.0 .0)
    }
}

impl From<Element> for Node {
    fn from(value: Element) -> Self {
        Node(value.0 .0)
    }
}

impl From<Fragment> for Node {
    fn from(value: Fragment) -> Self {
        Node(value.0 .0)
    }
}

impl Element {
    pub fn to_debug_html(&self) -> String {
        let mut buf = String::new();
        self.debug_html(&mut buf);
        buf
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NodeData {
    parent: Option<NodeId>,
    pub ty: NodeType,
}

trait DebugHtml {
    fn debug_html(&self, buf: &mut String);
}

impl DebugHtml for Element {
    fn debug_html(&self, buf: &mut String) {
        Document::with_node(self.0 .0, |node| {
            node.debug_html(buf);
        });
    }
}

impl DebugHtml for Text {
    fn debug_html(&self, buf: &mut String) {
        Document::with_node(self.0 .0, |node| {
            node.debug_html(buf);
        });
    }
}

impl DebugHtml for Node {
    fn debug_html(&self, buf: &mut String) {
        Document::with_node(self.0, |node| {
            node.debug_html(buf);
        });
    }
}

impl DebugHtml for NodeData {
    fn debug_html(&self, buf: &mut String) {
        match &self.ty {
            NodeType::Text(text) => buf.push_str(text),
            NodeType::Element {
                tag,
                attrs,
                children,
            } => {
                buf.push('<');
                buf.push_str(tag);
                for (k, v) in attrs {
                    buf.push(' ');
                    buf.push_str(k);
                    buf.push_str("=\"");
                    buf.push_str(v);
                    buf.push('"');
                }
                buf.push('>');

                for child in children {
                    child.debug_html(buf);
                }

                buf.push_str("</");
                buf.push_str(tag);
                buf.push('>');
            }
            NodeType::Fragment(nodes) => {
                for node in nodes {
                    node.debug_html(buf);
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Document(Rc<RefCell<SlotMap<NodeId, NodeData>>>);

impl Document {
    pub fn new() -> Self {
        Document(Default::default())
    }

    fn with_node<U>(id: NodeId, f: impl FnOnce(&NodeData) -> U) -> Option<U> {
        DOCUMENT.with(|d| {
            let data = d.0.borrow();
            let data = data.get(id);
            data.map(f)
        })
    }

    fn with_node_mut<U>(
        id: NodeId,
        f: impl FnOnce(&mut NodeData) -> U,
    ) -> Option<U> {
        DOCUMENT.with(|d| {
            let mut data = d.0.borrow_mut();
            let data = data.get_mut(id);
            data.map(f)
        })
    }

    pub fn reset(&self) {
        self.0.borrow_mut().clear();
    }

    fn create_element(&self, tag: &str) -> Element {
        Element(Node(self.0.borrow_mut().insert(NodeData {
            parent: None,
            ty: NodeType::Element {
                tag: tag.to_string().into(),
                attrs: HashMap::new(),
                children: Vec::new(),
            },
        })))
    }

    fn create_text_node(&self, data: &str) -> Text {
        Text(Node(self.0.borrow_mut().insert(NodeData {
            parent: None,
            ty: NodeType::Text(data.to_string()),
        })))
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static DOCUMENT: Document = Document::new();
}

pub fn document() -> Document {
    DOCUMENT.with(Clone::clone)
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeType {
    Text(String),
    Element {
        tag: Cow<'static, str>,
        attrs: HashMap<String, String>,
        children: Vec<Node>,
    },
    Fragment(Vec<Node>),
}

impl Mountable<MockDom> for Node {
    fn unmount(&mut self) {
        todo!()
    }

    fn mount(&self, parent: &Element, marker: Option<&Node>) {
        MockDom::insert_node(parent, self, marker);
    }
}

impl Mountable<MockDom> for Text {
    fn unmount(&mut self) {
        todo!()
    }

    fn mount(&self, parent: &Element, marker: Option<&Node>) {
        MockDom::insert_node(parent, self.as_ref(), marker);
    }
}

impl Mountable<MockDom> for Element {
    fn unmount(&mut self) {
        todo!()
    }

    fn mount(&self, parent: &Element, marker: Option<&Node>) {
        MockDom::insert_node(parent, self.as_ref(), marker);
    }
}

impl Mountable<MockDom> for Fragment {
    fn unmount(&mut self) {
        todo!()
    }

    fn mount(
        &self,
        parent: &<MockDom as Renderer>::Element,
        marker: Option<&<MockDom as Renderer>::Node>,
    ) {
        todo!()
    }
}

impl Renderer for MockDom {
    type Node = Node;
    type Text = Text;
    type Element = Element;

    fn create_element<E: ElementType>() -> Self::Element {
        document().create_element(E::TAG)
    }

    fn create_text_node(data: &str) -> Self::Text {
        document().create_text_node(data)
    }

    fn set_text(node: &Self::Text, text: &str) {
        Document::with_node_mut(node.0 .0, |node| {
            if let NodeType::Text(ref mut node) = node.ty {
                *node = text.to_string();
            }
        });
    }

    fn set_attribute(node: &Self::Element, name: &str, value: &str) {
        Document::with_node_mut(node.0 .0, |node| {
            if let NodeType::Element { ref mut attrs, .. } = node.ty {
                attrs.insert(name.to_string(), value.to_string());
            }
        });
    }

    fn remove_attribute(node: &Self::Element, name: &str) {
        Document::with_node_mut(node.0 .0, |node| {
            if let NodeType::Element { ref mut attrs, .. } = node.ty {
                attrs.remove(name);
            }
        });
    }

    fn insert_node(
        parent: &Self::Element,
        new_child: &Self::Node,
        anchor: Option<&Self::Node>,
    ) {
        debug_assert!(&parent.0 != new_child);
        // remove if already mounted
        if let Some(parent) = MockDom::get_parent(new_child) {
            let parent = Element(parent);
            MockDom::remove_node(&parent, new_child);
        }
        // mount on new parent
        Document::with_node_mut(parent.0 .0, |parent| {
            if let NodeType::Element {
                ref mut children, ..
            } = parent.ty
            {
                match anchor {
                    None => children.push(new_child.clone()),
                    Some(anchor) => {
                        let anchor_pos = children
                            .iter()
                            .position(|item| item.0 == anchor.0)
                            .expect("anchor is not a child of the parent");
                        children.insert(anchor_pos, new_child.clone());
                    }
                }
            } else {
                panic!("parent is not an element");
            }
        });
        // set parent on child node
        Document::with_node_mut(new_child.0, |node| {
            node.parent = Some(parent.0 .0)
        });
    }

    fn remove_node(
        parent: &Self::Element,
        child: &Self::Node,
    ) -> Option<Self::Node> {
        Document::with_node_mut(parent.0 .0, |parent| {
            if let NodeType::Element {
                ref mut children, ..
            } = parent.ty
            {
                let current_pos = children
                    .iter()
                    .position(|item| item.0 == child.0)
                    .expect("anchor is not a child of the parent");
                Some(children.remove(current_pos))
            } else {
                None
            }
        })
        .flatten()
    }

    fn remove(node: &Self::Node) {
        let parent = Element(Node(
            Self::get_parent(node)
                .expect("tried to remove a parentless node")
                .0,
        ));
        Self::remove_node(&parent, node);
    }

    fn get_parent(node: &Self::Node) -> Option<Self::Node> {
        Document::with_node(node.0, |node| node.parent)
            .flatten()
            .map(Node)
    }

    fn first_child(node: &Self::Node) -> Option<Self::Node> {
        Document::with_node(node.0, |node| match &node.ty {
            NodeType::Text(_) => None,
            NodeType::Element { children, .. } => children.get(0).cloned(),
            NodeType::Fragment(nodes) => nodes.get(0).cloned(),
        })
        .flatten()
    }

    fn next_sibling(node: &Self::Node) -> Option<Self::Node> {
        let node_id = node.0;
        Document::with_node(node_id, |node| {
            node.parent.and_then(|parent| {
                Document::with_node(parent, |parent| match &parent.ty {
                    NodeType::Text(_) => None,
                    NodeType::Element { children, .. } => {
                        let this = children
                            .iter()
                            .position(|check| check == &Node(node_id))?;
                        children.get(this + 1).cloned()
                    }
                    NodeType::Fragment(_) => todo!(),
                })
            })
        })
        .flatten()
        .flatten()
    }

    fn replace_node(old: &Self::Node, new: &Self::Node) {
        todo!()
    }
}

impl CastFrom<Node> for Text {
    fn cast_from(source: Node) -> Option<Self> {
        Document::with_node(source.0, |node| {
            matches!(node.ty, NodeType::Text(_))
        })
        .and_then(|matches| matches.then_some(Text(Node(source.0))))
    }
}

impl CastFrom<Node> for Element {
    fn cast_from(source: Node) -> Option<Self> {
        Document::with_node(source.0, |node| {
            matches!(node.ty, NodeType::Element { .. })
        })
        .and_then(|matches| matches.then_some(Element(Node(source.0))))
    }
}

#[cfg(test)]
mod tests {
    use super::MockDom;
    use crate::{
        html::element,
        renderer::{mock_dom::node_eq, Renderer},
    };

    #[test]
    fn html_debugging_works() {
        let main = MockDom::create_element::<element::Main>();
        let p = MockDom::create_element::<element::P>();
        MockDom::set_attribute(&p, "id", "foo");
        let text = MockDom::create_text_node("Hello, world!");
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&p, text.as_ref(), None);
        assert_eq!(
            main.to_debug_html(),
            "<main><p id=\"foo\">Hello, world!</p></main>"
        );
    }

    #[test]
    fn remove_attribute_works() {
        let main = MockDom::create_element::<element::Main>();
        let p = MockDom::create_element::<element::P>();
        MockDom::set_attribute(&p, "id", "foo");
        let text = MockDom::create_text_node("Hello, world!");
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&p, text.as_ref(), None);
        MockDom::remove_attribute(&p, "id");
        assert_eq!(main.to_debug_html(), "<main><p>Hello, world!</p></main>");
    }

    #[test]
    fn remove_node_works() {
        let main = MockDom::create_element::<element::Main>();
        let p = MockDom::create_element::<element::P>();
        MockDom::set_attribute(&p, "id", "foo");
        let text = MockDom::create_text_node("Hello, world!");
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&p, text.as_ref(), None);
        MockDom::remove_node(&main, p.as_ref());
        assert_eq!(main.to_debug_html(), "<main></main>");
    }

    #[test]
    fn insert_before_works() {
        let main = MockDom::create_element::<element::Main>();
        let p = MockDom::create_element::<element::P>();
        let span = MockDom::create_element::<element::Span>();
        let text = MockDom::create_text_node("Hello, world!");
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&span, text.as_ref(), None);
        MockDom::insert_node(&main, span.as_ref(), Some(p.as_ref()));
        assert_eq!(
            main.to_debug_html(),
            "<main><span>Hello, world!</span><p></p></main>"
        );
    }

    #[test]
    fn insert_before_sets_parent() {
        let main = MockDom::create_element::<element::Main>();
        let p = MockDom::create_element::<element::P>();
        MockDom::insert_node(&main, p.as_ref(), None);
        let parent =
            MockDom::get_parent(p.as_ref()).expect("p should have parent set");
        assert!(node_eq(parent, main));
    }

    #[test]
    fn insert_before_moves_node() {
        let main = MockDom::create_element::<element::Main>();
        let p = MockDom::create_element::<element::P>();
        let span = MockDom::create_element::<element::Span>();
        let text = MockDom::create_text_node("Hello, world!");
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&span, text.as_ref(), None);
        MockDom::insert_node(&main, span.as_ref(), Some(p.as_ref()));
        MockDom::insert_node(&main, p.as_ref(), Some(span.as_ref()));
        assert_eq!(
            main.to_debug_html(),
            "<main><p></p><span>Hello, world!</span></main>"
        );
    }

    #[test]
    fn first_child_gets_first_child() {
        let main = MockDom::create_element::<element::Main>();
        let p = MockDom::create_element::<element::P>();
        let span = MockDom::create_element::<element::Span>();
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&p, span.as_ref(), None);
        assert_eq!(
            MockDom::first_child(main.as_ref()).as_ref(),
            Some(p.as_ref())
        );
        assert_eq!(
            MockDom::first_child(&MockDom::first_child(main.as_ref()).unwrap())
                .as_ref(),
            Some(span.as_ref())
        );
    }

    #[test]
    fn next_sibling_gets_next_sibling() {
        let main = MockDom::create_element::<element::Main>();
        let p = MockDom::create_element::<element::P>();
        let span = MockDom::create_element::<element::Span>();
        let text = MockDom::create_text_node("foo");
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&main, span.as_ref(), None);
        MockDom::insert_node(&main, text.as_ref(), None);
        assert_eq!(
            MockDom::next_sibling(p.as_ref()).as_ref(),
            Some(span.as_ref())
        );
        assert_eq!(
            MockDom::next_sibling(span.as_ref()).as_ref(),
            Some(text.as_ref())
        );
    }
}
