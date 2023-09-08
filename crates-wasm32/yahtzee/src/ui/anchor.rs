use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlAnchorElement, Node};

#[derive(Clone)]
pub struct Anchor {
    anchor: HtmlAnchorElement,
}
impl Anchor {
    pub fn new(document: Document) -> Self {
        let element = document.create_element("a").unwrap_throw();
        let anchor = element.dyn_into::<HtmlAnchorElement>().unwrap_throw();
        Self { anchor }.with_class("anchor")
    }
    pub fn with_id(self, id: &str) -> Self {
        self.anchor.set_id(id);
        self
    }
    pub fn with_class(self, class: &str) -> Self {
        self.anchor.set_class_name(class);
        self
    }
    pub fn with_text(self, text: &str) -> Self {
        self.anchor.set_text(text).unwrap_throw();
        self
    }
    pub fn with_link(self, link: &str) -> Self {
        self.anchor.set_href(link);
        self
    }
    pub fn hide(&self) {
        self.anchor.set_hidden(true);
    }
    pub fn show(&self) {
        self.anchor.set_hidden(false);
    }
}
impl AsRef<Node> for Anchor {
    fn as_ref(&self) -> &Node {
        self.anchor.as_ref()
    }
}