use serde::Serialize;
use snafu::prelude::*;

use std::fmt::Debug;

use crate::error::*;

// We introduce a Debug bound so we can print the variable if it ever fails to serialize, which should not happen
pub fn json_or_yaml_output<T: Debug + Serialize>(output: &T, json: bool) -> Result<String, Error> {
    let output = if json {
        serde_json::to_string_pretty(output).context(OutputJsonSnafu {
            content: format!("{:#?}", output),
        })?
    } else {
        serde_yaml_ng::to_string(output).context(OutputYamlSnafu {
            content: format!("{:#?}", output),
        })?
    };

    Ok(output)
}
