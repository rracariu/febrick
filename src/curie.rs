use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sophia::iri::{AsIriRef, IriRef};
use sophia_api::prelude::Term;
use sophia_api::term::SimpleTerm;

use crate::namespaces::PrefixNamespaceMap;

/// Compact IRI
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Curie {
    pub prefix: String,
    pub local_name: String,
}

impl Curie {
    pub fn new(prefix: &str, name: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            local_name: name.to_string(),
        }
    }

    /// Convert CURIE to full IRI using the provided namespace map.
    pub fn to_iri(&self, prefixes: &PrefixNamespaceMap) -> Result<IriRef<String>> {
        let ns = prefixes
            .get_ns(&self.prefix)
            .ok_or_else(|| anyhow!("Missing prefix {}", self.prefix))?;
        self.to_iri_with_ns(ns)
    }

    fn to_iri_with_ns(&self, ns: &str) -> Result<IriRef<String>> {
        IriRef::new(format!("{}#{}", ns, self.local_name))
            .map_err(|e| anyhow!("Invalid IRI: {}", e))
    }

    /// Create a CURIE from an IRI.
    pub fn from_iri(iri: impl AsIriRef, prefixes: &PrefixNamespaceMap) -> Result<Self> {
        let iri = iri.as_iri_ref();
        let base = iri.as_base();
        let fragment = base.fragment();

        let local_name: String = if let Some(fragment) = fragment {
            if fragment.is_empty() {
                None
            } else {
                Some(fragment)
            }
        } else {
            let path = base.path();
            Some(&path[path.rfind('/').map_or(0, |i| i + 1)..])
        }
        .ok_or_else(|| anyhow!("Can't determine local name from, {iri}"))?
        .into();

        let prefix = &base[..base.len() - local_name.len()];

        let prefix = prefixes
            .get_prefix(prefix)
            .ok_or_else(|| anyhow!("Invalid prefix {prefix}"))?
            .to_string();

        Ok(Self { prefix, local_name })
    }

    pub fn from_term(term: &SimpleTerm, prefixes: &PrefixNamespaceMap) -> Result<Self> {
        if let Some(iri) = term.iri() {
            Self::from_iri(iri, prefixes)
        } else {
            Err(anyhow!("Term is not an IRI"))
        }
    }
}

impl TryFrom<&str> for Curie {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid CURIE format");
        }

        Ok(Self {
            prefix: parts[0].to_string(),
            local_name: parts[1].to_string(),
        })
    }
}

impl std::fmt::Display for Curie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.prefix, self.local_name)
    }
}

#[cfg(test)]
mod tests {
    use sophia::iri::Iri;

    use super::*;

    #[test]
    fn test_curie_to_iri() {
        let curie = Curie::new("brick", "Location");
        let iri = curie
            .to_iri_with_ns("https://brickschema.org/schema/Brick")
            .unwrap();
        assert_eq!(
            iri.as_str(),
            "https://brickschema.org/schema/Brick#Location"
        );

        let curie = Curie::new("ex", "MyEntity");
        let iri = curie.to_iri_with_ns("http://example.com").unwrap();
        assert_eq!(iri.as_str(), "http://example.com#MyEntity");
    }

    #[test]
    fn makes_curie_from_iri() {
        let prefixes = PrefixNamespaceMap::new(
            &[(
                "brick".to_string(),
                "https://brickschema.org/schema/Brick#".to_string(),
            )]
            .iter()
            .cloned()
            .collect(),
        );

        let iri = "https://brickschema.org/schema/Brick#Location";
        let curie = Curie::from_iri(Iri::new_unchecked(iri), &prefixes).unwrap();
        assert_eq!(curie.prefix, "brick");
        assert_eq!(curie.local_name, "Location");

        let prefixes = PrefixNamespaceMap::new(
            &[("sosa".to_string(), "http://www.w3.org/ns/sosa/".to_string())]
                .iter()
                .cloned()
                .collect(),
        );

        let iri = "http://www.w3.org/ns/sosa/FeatureOfInterest";
        let curie = Curie::from_iri(Iri::new_unchecked(iri), &prefixes).unwrap();
        assert_eq!(curie.prefix, "sosa");
        assert_eq!(curie.local_name, "FeatureOfInterest");
    }

    #[test]
    fn makes_curie_from_str() {
        let curie: Curie = "brick:Location".try_into().expect("Valid CURIE");
        assert_eq!(curie.prefix, "brick");
        assert_eq!(curie.local_name, "Location");

        let curie: Curie = "ex:MyEntity".try_into().expect("Valid CURIE");
        assert_eq!(curie.prefix, "ex");
        assert_eq!(curie.local_name, "MyEntity");

        let curie: Result<Curie> = "invalidcurie".try_into();
        assert!(curie.is_err());
    }
}
