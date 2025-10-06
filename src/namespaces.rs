use std::collections::HashMap;

use sophia_api::ns::Namespace;

/// A mapping between prefixes and namespaces.
#[derive(Debug, Clone)]
pub struct PrefixNamespaceMap {
    prefix_to_ns: HashMap<String, Namespace<String>>,
    ns_to_prefix: HashMap<String, String>,
}

impl PrefixNamespaceMap {
    pub fn new(prefix_map: &HashMap<String, String>) -> Self {
        let mut prefix_to_ns = HashMap::new();
        let mut ns_to_prefix = HashMap::new();

        for (prefix, ns) in prefix_map {
            prefix_to_ns.insert(prefix.clone(), Namespace::new_unchecked(ns.clone()));
            ns_to_prefix.insert(ns.clone(), prefix.clone());
        }

        Self {
            prefix_to_ns,
            ns_to_prefix,
        }
    }

    pub fn get_ns(&self, prefix: &str) -> Option<&Namespace<String>> {
        self.prefix_to_ns.get(prefix)
    }

    pub fn get_prefix(&self, ns: &str) -> Option<&String> {
        self.ns_to_prefix.get(ns)
    }
}
