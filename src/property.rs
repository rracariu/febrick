// Copyright (C) 2025 Radu Racariu.

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::curie::Curie;

#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[derive(Debug, Serialize, Deserialize)]
pub enum PropertyPairConstraint {
    Equal(String),
    Disjoint(String),
    LessThan(String),
    LessThanOrEqual(String),
}

#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[derive(Debug, Serialize, Deserialize)]
pub enum LogicalConstraint {
    Not(Vec<BrickProperty>),
    And(Vec<BrickProperty>),
    Or(Vec<BrickProperty>),
    XOne(Vec<BrickProperty>),
}

#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrickProperty {
    pub path: String,
    pub definition: String,
    pub class: Curie,
    pub subclass_of: Vec<Curie>,

    pub min_count: Option<u32>,
    pub max_count: Option<u32>,

    pub min_length: Option<u32>,
    pub max_length: Option<u32>,

    pub min_inclusive: Option<f64>,
    pub max_inclusive: Option<f64>,
    pub min_exclusive: Option<f64>,
    pub max_exclusive: Option<f64>,

    pub pattern: Option<String>,
    pub datatype: Option<Curie>,
    pub node_kind: Option<Curie>,

    pub constraints: Vec<PropertyPairConstraint>,
    pub logical_constraints: Vec<LogicalConstraint>,
    pub one_of: Vec<String>,
    pub has_value_of: String,
}
