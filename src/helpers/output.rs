use serde::Serialize;
use snafu::prelude::*;

use std::fmt::Debug;
use std::process::exit;

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

pub fn print_error_sources<E: std::error::Error>(error: E) {
    if let Some(source) = error.source() {
        print_error_sources(source);
    }

    eprintln!("{error}");
}

pub fn fallible_output<T: Debug + std::fmt::Display + Serialize>(
    output: Result<T, Error>,
    json: bool,
) {
    match output {
        Ok(output) => match json_or_yaml_output(&output, json) {
            Ok(serialized) => {
                println!("{}", serialized);
            }
            Err(e) => {
                eprintln!("{}", e);
                error!("An error occured, see backtrace above.");
                exit(1)
            }
        },
        Err(e) => {
            print_error_sources(e);
            // eprintln!("{}", e);
            error!("An error occured, see above.");
            exit(1)
        }
    }
}
