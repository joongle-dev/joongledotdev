use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlInputElement, KeyboardEvent, Node};

pub struct TextInput {
    input: HtmlInputElement,
}
impl TextInput {
    pub fn new(document: Document) -> Self {
        let element = document.create_element("input").unwrap_throw();
        let input = element.dyn_into::<HtmlInputElement>().unwrap_throw();
            input.set_type("text");
        Self { input }.with_class("input")
    }
    pub fn with_callback<F: FnMut(String) + 'static>(mut self, mut onenter: F) -> Self {
        let input_clone = self.input.clone();
        let callback = Closure::new(move |event: KeyboardEvent| {
            if event.key().eq("Enter") {
                onenter(input_clone.value());
            }
        });
        self.input.set_onkeydown(Some(callback.as_ref().unchecked_ref()));
        callback.forget();
        self
    }
    pub fn with_id(self, id: &str) -> Self {
        self.input.set_id(id);
        self
    }
    pub fn with_class(self, class: &str) -> Self {
        self.input.set_class_name(class);
        self
    }
    pub fn value(&self) -> String {
        self.input.value()
    }
    pub fn set_value(&self, value: &str) {
        self.input.set_value(value);
    }
    pub fn hide(&self) {
        self.input.set_hidden(true);
    }
    pub fn show(&self) {
        self.input.set_hidden(false);
    }
}
impl AsRef<Node> for TextInput {
    fn as_ref(&self) -> &Node {
        self.input.as_ref()
    }
}
