//! The calls the page makes, converting between JavaScript values and the Rust API. Maps cross as plain
//! objects so the page reads a table as it would read JSON.

use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

use crate::{Configuration, Entry, Table};

fn to_js(value: &impl Serialize) -> Result<JsValue, JsError> {
    value
        .serialize(&Serializer::json_compatible())
        .map_err(|error| JsError::new(&error.to_string()))
}

fn from_js<T: serde::de::DeserializeOwned>(value: JsValue) -> Result<T, JsError> {
    serde_wasm_bindgen::from_value(value).map_err(|error| JsError::new(&error.to_string()))
}

#[wasm_bindgen(js_name = check)]
pub fn check_js(kind: JsValue, text: &str) -> Result<JsValue, JsError> {
    let kind: Configuration = from_js(kind)?;
    to_js(&crate::check(kind, text).map_err(|reason| JsError::new(&reason))?)
}

#[wasm_bindgen(js_name = read)]
pub fn read_js(text: &str) -> Result<JsValue, JsError> {
    to_js(&crate::read(text).map_err(|reason| JsError::new(&reason))?)
}

#[wasm_bindgen(js_name = render)]
pub fn render_js(table: JsValue) -> Result<String, JsError> {
    let table: Table = from_js(table)?;
    Ok(crate::render(&table))
}

#[wasm_bindgen(js_name = stages)]
pub fn stages_js() -> Result<JsValue, JsError> {
    to_js(&crate::stages())
}

#[wasm_bindgen(js_name = simulate)]
pub fn simulate_js(config: &str, watch: &str, entries: JsValue) -> Result<JsValue, JsError> {
    let entries: Vec<Entry> = from_js(entries)?;
    to_js(&crate::simulate(config, watch, entries).map_err(|reason| JsError::new(&reason))?)
}
