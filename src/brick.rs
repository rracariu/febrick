// Copyright (c) 2024, Radu Racariu.

use anyhow::{anyhow, Result};

use sophia::inmem::graph::FastGraph;
use sophia_api::ns::Namespace;
use sophia_api::term::SimpleTerm;
use sophia_api::{
    graph::{Graph, MutableGraph},
    ns::{rdf, rdfs},
    prelude::TripleParser,
    term::matcher::Any,
    term::Term,
    triple::Triple,
};

use rio_api::parser::TriplesParser;
use sophia_rio::model::Trusted;
use sophia_turtle::parser::turtle::TurtleParser;

use crate::curie::Curie;
use crate::entity::BrickEntity;
use crate::namespaces::PrefixNamespaceMap;
use crate::property::{BrickProperty, LogicalConstraint};

pub struct Brick {
    graph: FastGraph,
    prefixes: PrefixNamespaceMap,
}

impl Brick {
    pub fn new(input: &str) -> Result<Self> {
        let mut graph = FastGraph::new();

        let mut parser = TurtleParser { base: None }.parse_str(input).0;

        parser.parse_all(&mut |triple| {
            graph.insert_triple(Trusted(triple))?;
            anyhow::Ok(())
        })?;

        let prefixes = PrefixNamespaceMap::new(parser.prefixes());

        Ok(Brick { graph, prefixes })
    }

    pub fn sub_class_of(&self, curie: &Curie) -> Result<Vec<Curie>> {
        let class = self.get_ns(&curie.prefix)?.get(&curie.local_name)?;

        self.graph
            .triples_matching(Any, [&rdfs::subClassOf], [&class])
            .flat_map(|triple| triple.map(|tr| Curie::from_term(tr.s(), &self.prefixes)))
            .collect()
    }

    pub fn super_class_of(&self, curie: &Curie) -> Result<Vec<Curie>> {
        let class = self.get_ns(&curie.prefix)?.get(&curie.local_name)?;

        self.graph
            .triples_matching([&class], [&rdfs::subClassOf], Any)
            .flat_map(|triple| triple.map(|tr| Curie::from_term(tr.o(), &self.prefixes)))
            .collect()
    }

    pub fn class_tags(&self, curie: &Curie) -> Result<Vec<String>> {
        let class = self.get_ns(&curie.prefix)?.get(&curie.local_name)?;

        self.graph
            .triples_matching(
                [&class],
                [&self.get_ns("brick")?.get("hasAssociatedTag")?],
                Any,
            )
            .flat_map(|triple| triple.map(|tr| without_prefix(tr.o())))
            .collect()
    }

    pub fn class_properties(&self, curie: &Curie) -> Result<Vec<BrickProperty>> {
        let class = self.get_ns(&curie.prefix)?.get(&curie.local_name)?;
        let prop = self.get_ns("sh")?.get("property")?;

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
                cur_prop = prop_term
                    .bnode_id()
                    .ok_or_else(|| anyhow!("Expecting Blank Node"))?
                    .to_string();
            }

            if let Some(prop) = props.last_mut() {
                let val = triple.p().iri().ok_or_else(|| anyhow!("Expecting IRI"))?;
                let base = val.as_base();
                let fragment = base.fragment().ok_or_else(|| anyhow!("Missing fragment"))?;

                if fragment == "message" {
                    prop.definition = triple
                        .o()
                        .lexical_form()
                        .ok_or_else(|| anyhow!("Expecting literal"))?
                        .to_string();
                } else if fragment == "class" {
                    prop.class = Curie::from_term(triple.o(), &self.prefixes)?;
                } else if fragment == "datatype" {
                    let iri = triple.o().iri().ok_or_else(|| anyhow!("Expecting IRI"))?;
                    let curie = Curie::from_iri(iri, &self.prefixes)?;

                    prop.datatype = Some(curie);
                } else if fragment == "nodeKind" {
                    let iri = triple.o().iri().ok_or_else(|| anyhow!("Expecting IRI"))?;
                    let curie = Curie::from_iri(iri, &self.prefixes)?;

                    prop.node_kind = Some(curie);
                } else if fragment == "path" {
                    prop.path = without_prefix(triple.o())?;
                } else if fragment == "not" {
                    prop.logical_constraints
                        .push(self.get_not_constraint(triple.o().clone())?);
                } else if fragment == "and" {
                    prop.logical_constraints
                        .push(self.get_and_constraint(triple.o().clone())?);
                } else if fragment == "or" {
                    prop.logical_constraints
                        .push(self.get_or_constraint(triple.o().clone())?);
                } else if fragment == "xone" {
                    prop.logical_constraints
                        .push(self.get_xone_constraint(triple.o().clone())?);
                }
            }
        }

        Ok(())
    }

    pub fn class_desc(&self, curie: &Curie) -> Result<BrickEntity> {
        let class = self.get_ns(&curie.prefix)?.get(&curie.local_name)?;

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
            .triples_matching([&class], [&self.get_ns("skos")?.get("definition")?], Any)
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

        let super_classes = self.super_class_of(curie)?;
        let tags = self.class_tags(curie)?;

        let properties = self.class_properties(curie)?;

        Ok(BrickEntity {
            name: curie.local_name.to_string(),
            namespace: curie.prefix.to_string(),
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

    fn get_ns(&self, prefix: &str) -> Result<&Namespace<String>> {
        self.prefixes
            .get_ns(prefix)
            .ok_or_else(|| anyhow!("Missing prefix {}", prefix))
    }
}

fn without_prefix(term: &SimpleTerm) -> Result<String> {
    let val = term.iri().ok_or_else(|| anyhow!("Expecting IRI"))?;
    let base = val.as_base();
    let fragment = base.fragment().ok_or_else(|| anyhow!("Missing fragment"))?;

    Ok(fragment.to_string())
}

fn only_prefix(term: &SimpleTerm) -> String {
    let val = term.iri().unwrap();

    let idx = val.rfind('#').map(|v| v + 1).unwrap_or_default();
    let begin = val[..idx].rfind('/').map(|v| v + 1).unwrap_or(idx);

    val[begin..].to_string()
}

#[cfg(test)]
mod test {

    use crate::{
        brick::{Brick, LogicalConstraint},
        curie::Curie,
    };
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
            .sub_class_of(&"brick:Point".try_into().unwrap())
            .unwrap()
            .contains(&Curie::try_from("brick:Sensor").unwrap()));
    }

    #[test]
    fn test_super_class_of() {
        let brick = ensure_brick();

        assert!(brick
            .super_class_of(&"brick:Sensor".try_into().unwrap())
            .unwrap()
            .contains(&"brick:Point".try_into().unwrap()));

        assert!(brick
            .super_class_of(&"brick:Capacity_Sensor".try_into().unwrap())
            .unwrap()
            .contains(&"brick:Sensor".try_into().unwrap()));

        assert!(brick
            .super_class_of(&"brick:Point".try_into().unwrap())
            .unwrap()
            .contains(&"brick:Entity".try_into().unwrap()));
    }

    #[test]
    fn test_class_tags() {
        let brick = ensure_brick();

        assert!(brick
            .class_tags(&Curie::new("brick", "Setpoint"))
            .unwrap()
            .contains(&"Point".to_string()));
    }

    #[test]
    fn test_class_desc() {
        let brick = ensure_brick();

        let desc = brick.class_desc(&Curie::new("brick", "Setpoint")).unwrap();

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
        assert_eq!(desc.super_classes, vec!["brick:Point".try_into().unwrap()]);
        assert_eq!(desc.tags, vec!["Point".to_string(), "Setpoint".to_string()]);
    }

    #[test]
    fn test_class_props() {
        let brick = ensure_brick();

        let props = brick
            .class_properties(&"brick:Location".try_into().unwrap())
            .unwrap();
        assert!(!props.is_empty());

        assert!(props
            .iter()
            .any(|p| p.path == "hasPoint" && p.class == Curie::new("brick", "Point")));

        assert!(props.iter().any(|p| p.path == "isPartOf"
            && p.logical_constraints.iter().any(
                |prop| matches!(prop, LogicalConstraint::Or(el) if el[0].class == Curie::new("brick","Location"))
            )));

        let props = brick
            .class_properties(&Curie::try_from("brick:Site").unwrap())
            .unwrap();
        assert!(!props.is_empty());

        assert!(props.iter().any(|p| p.path == "hasPart"
            && !p.logical_constraints.is_empty()
            && matches!(&p.logical_constraints[0], LogicalConstraint::Or(el) if el[0].class == Curie::new("brick","Building"))));
    }
}
