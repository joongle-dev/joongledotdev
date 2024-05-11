use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct TestStruct(u32);

#[wasm_bindgen]
impl TestStruct {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(0)
    }
    
    pub fn count(&mut self) -> u32 {
        let val = self.0;
        self.0 += 1;
        val
    }
}