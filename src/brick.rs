// Copyright (c) 2024, Radu Racariu.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sophia::{inmem::graph::FastGraph, turtle::parser::turtle};
use sophia_api::prelude::{Any, TripleSource};
use sophia_api::term::SimpleTerm;
use sophia_api::{
    graph::Graph,
    ns::{rdf, rdfs, Namespace},
    term::Term,
    triple::Triple,
};
use std::fmt::Debug;
use std::sync::LazyLock;

use crate::property::{BrickProperty, LogicalConstraint};

pub static BRICK_NS: LazyLock<Namespace<&'static str>> =
    LazyLock::new(|| Namespace::new_unchecked("https://brickschema.org/schema/Brick#"));
pub static SKOS_NS: LazyLock<Namespace<&'static str>> =
    LazyLock::new(|| Namespace::new_unchecked("http://www.w3.org/2004/02/skos/core#"));
pub static SHACL_NS: LazyLock<Namespace<&'static str>> =
    LazyLock::new(|| Namespace::new_unchecked("http://www.w3.org/ns/shacl#"));

#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BrickClass {
    pub name: String,
    pub label: String,
    pub definition: String,
    pub types: Vec<String>,
    pub super_classes: Vec<String>,
    pub tags: Vec<String>,
    pub properties: Vec<BrickProperty>,
}

pub struct Brick {
    graph: FastGraph,
}

impl Brick {
    pub fn new(input: &str) -> Result<Self> {
        let parser = turtle::parse_str(input);

        let graph: FastGraph = parser.collect_triples()?;

        Ok(Brick { graph })
    }

    pub fn sub_class_of(&self, class: &str) -> Result<Vec<String>> {
        let class = BRICK_NS.get(class)?;

        self.graph
            .triples_matching(Any, [&rdfs::subClassOf], [&class])
            .map(|triple| triple.map(|tr| without_ns(tr.s())).map_err(Into::into))
            .collect()
    }

    pub fn super_class_of(&self, class: &str) -> Result<Vec<String>> {
        let class = BRICK_NS.get(class)?;

        self.graph
            .triples_matching([&class], [&rdfs::subClassOf], Any)
            .map(|triple| triple.map(|tr| without_ns(tr.o())).map_err(Into::into))
            .collect()
    }

    pub fn class_tags(&self, class: &str) -> Result<Vec<String>> {
        let class = BRICK_NS.get(class)?;

        self.graph
            .triples_matching([&class], [&BRICK_NS.get("hasAssociatedTag")?], Any)
            .map(|triple| triple.map(|tr| without_ns(tr.o())).map_err(Into::into))
            .collect()
    }

    pub fn class_properties(&self, class_name: &str) -> Result<Vec<BrickProperty>> {
        let class = BRICK_NS.get(class_name)?;
        let prop = SHACL_NS.get("property")?;

        let mut props = Vec::<BrickProperty>::new();

        for prop_term in self
            .graph
            .triples_matching([&class], [&prop], Any)
            .map(|triple| triple.map(|el| el.o().clone()))
        {
            let prop_term = prop_term?;
            self.collect_props(prop_term, &mut props)?;
        }

        Ok(props)
    }

    fn collect_props(
        &self,
        prop_term: SimpleTerm,
        props: &mut Vec<BrickProperty>,
    ) -> Result<(), anyhow::Error> {
        let mut cur_prop = String::default();

        for triple in self.graph.triples_matching([prop_term.clone()], Any, Any) {
            let triple = triple?;

            if triple.s().bnode_id().is_some_and(|val| **val != cur_prop) {
                props.push(BrickProperty::default());
                cur_prop = prop_term.bnode_id().unwrap().to_string();
            }

            if let Some(prop) = props.last_mut() {
                let val = triple.p().iri().unwrap();

                if val.ends_with("#message") {
                    prop.definition = triple.o().lexical_form().unwrap().to_string();
                } else if val.ends_with("#class") {
                    prop.class = without_ns(triple.o());
                } else if val.ends_with("#path") {
                    prop.path = without_ns(triple.o());
                } else if val.ends_with("#not") {
                    prop.logical_constraints
                        .push(self.get_not_constraint(triple.o().clone())?);
                } else if val.ends_with("#and") {
                    prop.logical_constraints
                        .push(self.get_and_constraint(triple.o().clone())?);
                } else if val.ends_with("#or") {
                    prop.logical_constraints
                        .push(self.get_or_constraint(triple.o().clone())?);
                } else if val.ends_with("#xone") {
                    prop.logical_constraints
                        .push(self.get_xone_constraint(triple.o().clone())?);
                }
            }
        }

        Ok(())
    }

    pub fn class_desc(&self, class_name: &str) -> Result<BrickClass> {
        let class = BRICK_NS.get(class_name)?;

        let label = self
            .graph
            .triples_matching([&class], [&rdfs::label], Any)
            .map(|triple| {
                triple
                    .map(|tr| {
                        tr.o()
                            .lexical_form()
                            .map_or(String::new(), |v| v.to_string())
                    })
                    .map_err(Into::into)
            })
            .collect::<Result<String>>()?;

        let definition = self
            .graph
            .triples_matching([&class], [&SKOS_NS.get("definition")?], Any)
            .map(|triple| {
                triple
                    .map(|tr| {
                        tr.o()
                            .lexical_form()
                            .map_or(String::new(), |v| v.to_string())
                    })
                    .map_err(Into::into)
            })
            .collect::<Result<String>>()?;

        let types = self
            .graph
            .triples_matching([&class], [&rdf::type_], Any)
            .map(|triple| triple.map(|tr| only_prefix(tr.o())).map_err(Into::into))
            .collect::<Result<Vec<String>>>()?;

        let super_classes = self.super_class_of(class_name)?;
        let tags = self.class_tags(class_name)?;

        let properties = self.class_properties(class_name)?;

        Ok(BrickClass {
            name: class_name.to_string(),
            label,
            definition,
            types,
            super_classes,
            tags,
            properties,
        })
    }

    fn get_not_constraint(&self, or_term: SimpleTerm) -> Result<LogicalConstraint> {
        let props = self.collect_prop_list(or_term)?;
        Ok(LogicalConstraint::Not(props))
    }

    fn get_and_constraint(&self, or_term: SimpleTerm) -> Result<LogicalConstraint> {
        let props = self.collect_prop_list(or_term)?;
        Ok(LogicalConstraint::And(props))
    }

    fn get_or_constraint(&self, or_term: SimpleTerm) -> Result<LogicalConstraint> {
        let props = self.collect_prop_list(or_term)?;
        Ok(LogicalConstraint::Or(props))
    }

    fn get_xone_constraint(&self, or_term: SimpleTerm) -> Result<LogicalConstraint> {
        let props = self.collect_prop_list(or_term)?;
        Ok(LogicalConstraint::XOne(props))
    }

    fn collect_prop_list(
        &self,
        mut list_term: SimpleTerm,
    ) -> Result<Vec<BrickProperty>, anyhow::Error> {
        let mut props = Vec::<BrickProperty>::new();
        'outer: loop {
            for list_triple in self.graph.triples_matching([&list_term.clone()], Any, Any) {
                let list_triple = list_triple?;

                if list_triple
                    .p()
                    .iri()
                    .is_some_and(|iri| iri.ends_with("#rest"))
                {
                    list_term = list_triple.o().to_owned();
                    continue 'outer;
                }
                if list_triple
                    .p()
                    .iri()
                    .is_some_and(|iri| iri.ends_with("#first"))
                {
                    for prop_triple in self.graph.triples_matching([list_triple.o()], Any, Any) {
                        let prop_triple = prop_triple?;
                        self.collect_props(prop_triple.s().to_owned(), &mut props)?;
                    }
                } else {
                    break 'outer;
                }
            }
            break;
        }
        Ok(props)
    }
}

fn without_ns(term: &SimpleTerm) -> String {
    let val = term.iri().unwrap();
    val[val.rfind('#').map(|v| v + 1).unwrap_or_default()..].to_string()
}

fn only_prefix(term: &SimpleTerm) -> String {
    let val = term.iri().unwrap();

    let idx = val.rfind('#').map(|v| v + 1).unwrap_or_default();
    let begin = val[..idx].rfind('/').map(|v| v + 1).unwrap_or(idx);

    val[begin..].to_string()
}

#[cfg(test)]
mod test {

    use crate::brick::{Brick, LogicalConstraint};
    use std::io::prelude::*;

    fn ensure_brick() -> Brick {
        let mut file = std::fs::File::open("./Brick.ttl").unwrap();

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();

        Brick::new(&buffer).unwrap()
    }

    #[test]
    fn test_sub_class_of() {
        let brick = ensure_brick();

        assert!(brick
            .sub_class_of("Point")
            .unwrap()
            .contains(&"Sensor".to_string()));
    }

    #[test]
    fn test_super_class_of() {
        let brick = ensure_brick();

        assert!(brick
            .super_class_of("Sensor")
            .unwrap()
            .contains(&"Point".to_string()));

        assert!(brick
            .super_class_of("Capacity_Sensor")
            .unwrap()
            .contains(&"Sensor".to_string()));

        assert!(brick
            .super_class_of("Point")
            .unwrap()
            .contains(&"Entity".to_string()));
    }

    #[test]
    fn test_class_tags() {
        let brick = ensure_brick();

        assert!(brick
            .class_tags("Setpoint")
            .unwrap()
            .contains(&"Point".to_string()));
    }

    #[test]
    fn test_class_desc() {
        let brick = ensure_brick();

        let desc = brick.class_desc("Setpoint").unwrap();

        assert_eq!(desc.name, "Setpoint");
        assert_eq!(desc.tags, ["Point", "Setpoint"]);
        assert_eq!(
            desc.definition,
            "A Setpoint is an input value at which the desired property is set"
        );
        assert_eq!(
            desc.types,
            ["shacl#NodeShape".into(), "owl#Class".to_string()]
        );
        assert_eq!(desc.super_classes, vec!["Point".to_string()]);
        assert_eq!(desc.tags, vec!["Point".to_string(), "Setpoint".to_string()]);
    }

    #[test]
    fn test_class_props() {
        let brick = ensure_brick();

        let props = brick.class_properties("Location").unwrap();
        assert!(!props.is_empty());

        assert!(props
            .iter()
            .any(|p| p.path == "hasPoint" && p.class == "Point"));

        assert!(props.iter().any(|p| p.path == "isPartOf"
            && p.logical_constraints.iter().any(
                |prop| matches!(prop, LogicalConstraint::Or(el) if el[0].class == "Location")
            )));

        let props = brick.class_properties("Site").unwrap();
        assert!(!props.is_empty());

        assert!(props.iter().any(|p| p.path == "hasPart"
            && !p.logical_constraints.is_empty()
            && matches!(&p.logical_constraints[0], LogicalConstraint::Or(el) if el[0].class == "Building")));
    }
}
