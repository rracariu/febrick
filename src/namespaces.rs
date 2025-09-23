use std::{collections::HashMap, sync::OnceLock};

use anyhow::Result;
use sophia_api::ns::Namespace;

/// Global static map of namespaces initialized once.
/// This is initialized from the prefixes provided when loading the Brick schema.
pub static NS: OnceLock<HashMap<String, Namespace<String>>> = OnceLock::new();

/// Get a namespace by its prefix.
pub fn get_ns(ns: &str) -> Result<&Namespace<String>> {
    NS.get()
        .and_then(|map| map.get(ns))
        .ok_or_else(|| anyhow::anyhow!("Namespace not found: {}", ns))
}

pub(crate) fn init_ns_from_prefixes(prefixes: &HashMap<String, String>) {
    NS.get_or_init(|| {
        let mut map = HashMap::new();
        for (k, v) in prefixes {
            map.insert(k.clone(), Namespace::new_unchecked(v.clone()));
        }

        // Ensure some common namespaces are always present.
        map.insert(
            "shacl".into(),
            Namespace::new_unchecked("http://www.w3.org/ns/shacl#".to_string()),
        );

        map
    });
}
