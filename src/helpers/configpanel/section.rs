pub struct Section {
    // Inherited from container
    id: String,
    name: Option<String>,
    services: Vec<String>,
    help: Option<String>,

    // Section-pecific
    optional: bool,
    visible: bool,
    
    #[serde(flatten)]
    attrs: Table,
}

impl Section {
    fn visible -> bool {
        // TODO
        true
    }
}
