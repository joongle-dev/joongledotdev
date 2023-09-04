use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlButtonElement, MouseEvent, Node};

pub struct Button {
    button: HtmlButtonElement,
}
impl AsRef<Node> for Button {
    fn as_ref(&self) -> &Node {
        self.button.as_ref()
    }
}
impl Button {
    pub fn new(document: Document) -> Self {
        let element = document.create_element("button").unwrap_throw();
        let button = element.dyn_into::<HtmlButtonElement>().unwrap_throw();
        Self { button }.with_class("button")
    }
    pub fn with_text(self, text: &str) -> Self {
        self.button.set_text_content(Some(text));
        self
    }
    pub fn with_callback<F: FnMut() + 'static>(self, mut onclick: F) -> Self {
        let callback = Closure::new(move |_: MouseEvent| { onclick(); });
        self.button.set_onclick(Some(callback.as_ref().unchecked_ref()));
        callback.forget();
        self
    }
    pub fn with_id(self, id: &str) -> Self {
        self.button.set_id(id);
        self
    }
    pub fn with_class(self, class: &str) -> Self {
        self.button.set_class_name(class);
        self
    }
    pub fn hide(&self) {
        self.button.set_hidden(true);
    }
    pub fn show(&self) {
        self.button.set_hidden(false);
    }
}