use wasm_bindgen::JsValue;
use web_sys::{AbortController, AbortSignal};

pub struct Abort {
    controller: AbortController,
}

impl Abort {
    pub fn new() -> Result<Self, JsValue> {
        Ok(Self {
            controller: AbortController::new()?,
        })
    }

    pub fn signal(&self) -> AbortSignal {
        self.controller.signal()
    }
}

impl Drop for Abort {
    fn drop(&mut self) {
        self.controller.abort();
    }
}