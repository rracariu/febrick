use serde::{Deserialize, Serialize};

use crate::{curie::Curie, property::BrickProperty};

#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrickEntity {
    pub name: String,
    pub namespace: String,
    pub label: String,
    pub definition: String,
    pub types: Vec<String>,
    pub super_classes: Vec<Curie>,
    pub tags: Vec<String>,
    pub properties: Vec<BrickProperty>,
}
