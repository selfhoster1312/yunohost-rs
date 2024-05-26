use toml::{Table, Value};

use super::Map;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppliedClassicContainer {
    #[serde(flatten)]
    pub fields: Map<String, AppliedClassicValue>,
}

impl AppliedClassicContainer {
    pub fn new() -> Self {
        Self { fields: Map::new() }
    }
}

/// Once we have applied settings and translated stuff, only ask/value remain in the compact view.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppliedClassicValue {
    pub ask: String,
    // TODO: why is everything always a string???
    // Actually, for type="alert", we have a "ask" but no value
    pub value: Option<String>,
}

impl AppliedClassicValue {
    pub fn new(ask: String, value: Option<String>) -> Self {
        Self { ask, value }
    }
}

impl AppliedClassicValue {
    pub fn to_toml_value(&self) -> Value {
        // UNWRAP NOTE: This struct is very straightforward so (de)serialization should not fail
        Value::Table(Table::try_from(&self).unwrap())
    }
}
