use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlDivElement, Node};
use crate::ui::{anchor::Anchor, button::Button, text_input::TextInput};

pub struct Div {
    document: Document,
    div: HtmlDivElement,
}
impl Div {
    pub fn new(document: Document) -> Self {
        let element = document.create_element("div").unwrap_throw();
        let div = element.dyn_into::<HtmlDivElement>().unwrap_throw();
        Self { document, div }
    }
    pub fn with_id(self, id: &str) -> Self {
        self.div.set_id(id);
        self
    }
    pub fn with_class(self, class: &str) -> Self {
        self.div.set_class_name(class);
        self
    }
    pub fn clear(&self) {
        self.div.replace_children_with_node_0();
    }
    pub fn div(&self) -> Div {
        let div = Div::new(self.document.clone());
        self.div.append_child(div.as_ref()).unwrap_throw();
        div
    }
    pub fn text(&self, text: &str) {
        self.div.append_with_str_1(text).unwrap_throw();
    }
    pub fn anchor(&self) -> Anchor {
        let anchor = Anchor::new(self.document.clone());
        self.div.append_child(anchor.as_ref()).unwrap_throw();
        anchor
    }
    pub fn button(&self) -> Button {
        let button = Button::new(self.document.clone());
        self.div.append_child(button.as_ref()).unwrap_throw();
        button
    }
    pub fn text_input(&self) -> TextInput {
        let text_input = TextInput::new(self.document.clone());
        self.div.append_child(text_input.as_ref()).unwrap_throw();
        text_input
    }
    pub fn append_child<N: AsRef<Node>>(&self, node: &N) {
        self.div.append_child(node.as_ref()).unwrap_throw();
    }
    pub fn remove_child<N: AsRef<Node>>(&self, node: &N) {
        self.div.remove_child(node.as_ref()).unwrap_throw();
    }
    pub fn hide(&self) {
        self.div.set_hidden(true);
    }
    pub fn show(&self) {
        self.div.set_hidden(false);
    }
    pub fn remove(&self) {
        self.div.remove();
    }
}
impl AsRef<Node> for Div {
    fn as_ref(&self) -> &Node {
        self.div.as_ref()
    }
}
