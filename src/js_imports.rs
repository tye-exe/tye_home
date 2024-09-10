use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/assets/snippets.js")]
extern "C" {
    pub fn is_mobile() -> bool;
}
