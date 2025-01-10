use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::Element;

// https://github.com/fengyuanchen/viewerjs
#[wasm_bindgen]
extern "C" {

    pub type Viewer;

    // = new Viewer(document.getElementById("a-select"));
    #[wasm_bindgen(constructor)]
    pub fn new(_: &Element) -> Viewer;

    // = new Viewer(document.getElementById("a-select"), {url="data-original"});
    #[wasm_bindgen(constructor)]
    pub fn new_with_original(_: &Element, _: &JsValue) -> Viewer;

    #[wasm_bindgen(method)]
    pub fn update(this: &Viewer);

    #[wasm_bindgen(method)]
    pub fn destroy(this: &Viewer);
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerOption {
    pub url: String,
}

impl Default for ViewerOption {
    fn default() -> Self { 
        Self {
            url: String::from("data-original")
        }
    }
}

impl ViewerOption {
    pub fn to_value(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self).unwrap()
    }
}