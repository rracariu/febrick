// Copyright (c) 2024, Radu Racariu.

//!
//! This module provides a WebAssembly interface for the FeBrick crate.
//!

use crate::{brick::Brick as BrickImpl, curie::Curie};
use log::info;
use serde_wasm_bindgen::{from_value, to_value};
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
    #[wasm_bindgen(js_name = subClassOf, unchecked_return_type = "Curie[]")]
    pub fn sub_classes_of(
        &self,
        #[wasm_bindgen(unchecked_param_type = "Curie")] curie: JsValue,
    ) -> Result<Vec<JsValue>, String> {
        let curie = from_value::<Curie>(curie).map_err(|err| err.to_string())?;

        self.brick
            .sub_classes_of(&curie)
            .map_err(|err| err.to_string())
            .and_then(|vec| {
                vec.into_iter()
                    .map(|curie| to_value(&curie))
                    .collect::<Result<_, _>>()
                    .map_err(|err| err.to_string())
            })
    }

    /// For the given class, return all of its superclasses names.
    #[wasm_bindgen(js_name = superClassOf, unchecked_return_type = "Curie[]")]
    pub fn super_classes_of(
        &self,
        #[wasm_bindgen(unchecked_param_type = "Curie")] curie: JsValue,
    ) -> Result<Vec<JsValue>, String> {
        let curie = from_value(curie).map_err(|err| err.to_string())?;

        self.brick
            .super_classes_of(&curie)
            .map_err(|err| err.to_string())
            .and_then(|vec| {
                vec.into_iter()
                    .map(|curie| to_value(&curie))
                    .collect::<Result<_, _>>()
                    .map_err(|err| err.to_string())
            })
    }

    /// For the given class, return all of its tags.
    #[wasm_bindgen(js_name = classTags)]
    pub fn class_tags(
        &self,
        #[wasm_bindgen(unchecked_param_type = "Curie")] curie: JsValue,
    ) -> Result<Vec<String>, String> {
        let curie = from_value(curie).map_err(|err| err.to_string())?;

        Ok(self
            .brick
            .class_tags(&curie)
            .map_err(|err| err.to_string())?)
    }

    /// For the given class curie, return its core definition.
    #[wasm_bindgen(js_name = classDescription, unchecked_return_type = "BrickEntity")]
    pub fn class_description(
        &self,
        #[wasm_bindgen(unchecked_param_type = "Curie")] curie: JsValue,
    ) -> Result<JsValue, String> {
        let curie = from_value(curie).map_err(|err| err.to_string())?;

        self.brick
            .class_desc(&curie)
            .map_err(|err| err.to_string())
            .and_then(|vec| to_value(&vec).map_err(|err| err.to_string()))
    }

    /// For the given class, return all of its properties names.
    #[wasm_bindgen(js_name = classProperties, unchecked_return_type = "BrickProperty[]")]
    pub fn class_properties(
        &self,
        #[wasm_bindgen(unchecked_param_type = "Curie")] curie: JsValue,
    ) -> Result<JsValue, String> {
        let curie = from_value(curie).map_err(|err| err.to_string())?;

        self.brick
            .class_properties(&curie)
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
