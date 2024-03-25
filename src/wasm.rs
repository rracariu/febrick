// Copyright (c) 2024, Radu Racariu.

//!
//! This module provides a WebAssembly interface for the FeBrick crate.
//!

use crate::brick::Brick as BrickImpl;
use log::info;
use serde_wasm_bindgen::to_value;
use std::panic;
use wasm_bindgen::prelude::*;
use wasm_bindgen_console_logger::DEFAULT_LOGGER;

#[wasm_bindgen]
pub struct Brick {
    brick: BrickImpl,
}

#[wasm_bindgen]
impl Brick {
    /// Create a new Brick instance.
    /// # Arguments
    /// * `ttl` - A Turtle file content encoding the Brick Schema.
    #[wasm_bindgen(constructor)]
    pub fn new(ttl: &str) -> Result<Brick, String> {
        Ok(Brick {
            brick: BrickImpl::new(ttl).map_err(|err| err.to_string())?,
        })
    }

    /// For the given class, return all of its subclasses names.
    #[wasm_bindgen(js_name = subClassOf)]
    pub fn sub_class_of(&self, class: &str) -> Result<Vec<String>, String> {
        Ok(self
            .brick
            .sub_class_of(class)
            .map_err(|err| err.to_string())?)
    }

    /// For the given class, return all of its superclasses names.
    #[wasm_bindgen(js_name = superClassOf)]
    pub fn super_class_of(&self, class: &str) -> Result<Vec<String>, String> {
        Ok(self
            .brick
            .super_class_of(class)
            .map_err(|err| err.to_string())?)
    }

    /// For the given class, return all of its tags.
    #[wasm_bindgen(js_name = classTags)]
    pub fn class_tags(&self, class: &str) -> Result<Vec<String>, String> {
        Ok(self
            .brick
            .class_tags(class)
            .map_err(|err| err.to_string())?)
    }

    /// For the given class, return its core definition.
    #[wasm_bindgen(js_name = classDescription)]
    pub fn class_description(&self, class: &str) -> Result<JsValue, String> {
        self.brick
            .class_desc(class)
            .map_err(|err| err.to_string())
            .and_then(|vec| to_value(&vec).map_err(|err| err.to_string()))
    }

    /// For the given class, return all of its properties names.
    #[wasm_bindgen(js_name = classProperties)]
    pub fn class_properties(&self, class: &str) -> Result<JsValue, String> {
        self.brick
            .class_properties(class)
            .map_err(|err| err.to_string())
            .and_then(|vec| to_value(&vec).map_err(|err| err.to_string()))
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    log::set_logger(&DEFAULT_LOGGER).expect("Unable to set default logger.");

    #[cfg(debug_assertions)]
    log::set_max_level(log::LevelFilter::Debug);
    #[cfg(not(debug_assertions))]
    log::set_max_level(log::LevelFilter::Info);

    info!("Febrick module loaded.");
}
