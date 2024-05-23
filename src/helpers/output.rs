use serde::Serialize;
use snafu::prelude::*;

use std::fmt::Debug;
use std::process::exit;
use std::sync::OnceLock;

use crate::error::*;

static JSON_OUTPUT: OnceLock<bool> = OnceLock::new();

pub fn enable_json() {
    JSON_OUTPUT.get_or_init(|| true);
}

// We introduce a Debug bound so we can print the variable if it ever fails to serialize, which should not happen
pub fn format<T: Debug + Serialize>(output: &T) -> Result<String, Error> {
    // If JSON output was not requested yet, default to false
    let output = if *JSON_OUTPUT.get_or_init(|| false) {
        serde_json::to_string_pretty(output).context(OutputJsonSnafu {
            content: format!("{:#?}", output),
        })?
    } else {
        serde_yaml_ng::to_string(output).context(OutputYamlSnafu {
            content: format!("{:#?}", output),
        })?
    };

    Ok(output.to_string())
}

pub fn print_error_sources<E: std::error::Error>(error: E) {
    if let Some(source) = error.source() {
        print_error_sources(source);
    }

    eprintln!("{error}");
}

pub fn fallible<T: Debug + std::fmt::Display + Serialize>(output: Result<T, Error>) {
    match output {
        Ok(output) => match format(&output) {
            Ok(serialized) => {
                println!("{}", serialized);
            }
            Err(e) => {
                eprintln!("{}", e);
                error!("An error occured during formatting output, see above.");
                exit(1)
            }
        },
        Err(e) => {
            print_error_sources(e);
            error!("An error occured, see backtrace above.");
            exit(1)
        }
    }
}
