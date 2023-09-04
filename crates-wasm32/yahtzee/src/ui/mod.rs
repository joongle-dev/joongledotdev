pub mod div;
pub mod button;
pub mod textinput;
pub mod anchor;

use wasm_bindgen::prelude::*;
use web_sys::Document;
use div::Div;

pub struct Ui {
    document: Document,
    root: Div,
}
impl Ui {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let body = document.body().unwrap_throw();

        let root = Div::new(document.clone()).with_class("ui");
        body.append_child(root.as_ref()).unwrap_throw();

        Self { document, root }
    }
    pub fn div(&self) -> Div {
        self.root.div()
    }
}
impl Drop for Ui {
    fn drop(&mut self) {
        self.root.remove()
    }
}
