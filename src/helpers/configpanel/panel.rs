use std::collections::BTreeMap;

use super::Section;

pub struct Panel {
    // Inherited from container
    id: String,
    name: Option<String>,
    services: Vec<String>,
    help: Option<String>,

    actions: BTreeMap<String, String>,
    bind: Option<String>,
    sections: Vec<Section>,
    
    #[serde(flatten)]
    attrs: Table,
}
