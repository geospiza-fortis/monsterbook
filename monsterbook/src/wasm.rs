//! wasm-bindgen bindings for [`crate::session::Session`], exposed to the web
//! app as the `Session` class. Errors are plain JS objects
//! `{ kind: "NoPageFound" | "DecodeError", message }` so they survive
//! structured clone across postMessage.
use crate::session::{IngestError, Session};
use image::{DynamicImage, ImageOutputFormat};
use std::io::Cursor;
use wasm_bindgen::prelude::*;

fn js_error(kind: &str, message: &str) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&obj, &"kind".into(), &kind.into());
    let _ = js_sys::Reflect::set(&obj, &"message".into(), &message.into());
    obj.into()
}

fn ingest_error(e: IngestError) -> JsValue {
    let kind = match e {
        IngestError::DecodeError => "DecodeError",
        IngestError::NoPageFound => "NoPageFound",
    };
    js_error(kind, &e.to_string())
}

#[wasm_bindgen(js_name = Session)]
pub struct WasmSession {
    inner: Session,
}

#[wasm_bindgen(js_class = Session)]
impl WasmSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSession {
        WasmSession {
            inner: Session::new(),
        }
    }

    /// Ingest an encoded screenshot (PNG/JPEG/... bytes). Returns the page_id.
    pub fn add_screenshot(&mut self, bytes: &[u8]) -> Result<usize, JsValue> {
        self.inner.add_screenshot(bytes).map_err(ingest_error)
    }

    /// Ingest a raw RGBA bitmap. Returns the page_id.
    pub fn add_bitmap(&mut self, width: u32, height: u32, rgba: &[u8]) -> Result<usize, JsValue> {
        self.inner
            .add_bitmap(width, height, rgba.to_vec())
            .map_err(ingest_error)
    }

    /// Page ids not yet ingested.
    pub fn missing(&self) -> Vec<usize> {
        self.inner.missing()
    }

    /// Transcribe ingested pages: array of { uid, name, count }.
    pub fn transcribe(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.transcribe())
            .map_err(|e| js_error("InternalError", &e.to_string()))
    }

    /// Stitch ingested pages' cards into a PNG (encoded bytes). Empty
    /// (un-caught) card slots are skipped unless `include_empty` is set.
    pub fn stitch(&self, cards_per_row: u32, include_empty: bool) -> Result<Vec<u8>, JsValue> {
        let img = self
            .inner
            .stitch(cards_per_row, include_empty)
            .ok_or_else(|| js_error("NoPageFound", "no cards to stitch"))?;
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(img)
            .write_to(&mut bytes, ImageOutputFormat::Png)
            .map_err(|e| js_error("InternalError", &e.to_string()))?;
        Ok(bytes.into_inner())
    }
}

impl Default for WasmSession {
    fn default() -> Self {
        Self::new()
    }
}
