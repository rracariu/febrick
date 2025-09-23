use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sophia::iri::AsIriRef;

/// Compact IRI
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Curie {
    pub prefix: String,
    pub local_name: String,
}

impl Curie {
    pub fn from_iri(iri: impl AsIriRef) -> Result<Self> {
        let iri = iri.as_iri_ref();
        let base = iri.as_base();

        let prefix = base
            .path()
            .split('/')
            .next_back()
            .ok_or_else(|| anyhow!("Missing IRI prefix"))
            .and_then(|seg| {
                if seg.is_empty() {
                    Err(anyhow!("Empty IRI prefix"))
                } else {
                    Ok(seg)
                }
            })?
            .into();

        let local_name = base
            .fragment()
            .ok_or_else(|| anyhow!("Missing IRI fragment"))?
            .into();

        Ok(Self { prefix, local_name })
    }
}
