use serde::Serialize;
use snafu::prelude::*;

use std::fmt::Debug;
use std::sync::OnceLock;

use crate::error::*;

static JSON_OUTPUT: OnceLock<bool> = OnceLock::new();

pub fn enable_json() {
    JSON_OUTPUT.get_or_init(|| true);
}

/// Format the output in JSON or YAML.
///
/// The format is decided by the state of the [`JSON_OUTPUT`] setting, which is usually
/// set by CLI command when they receive a `--json` argument.
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

/// Prints an error and its sources, recursively.
pub fn print_error_sources<E: std::error::Error>(error: E) {
    if let Some(source) = error.source() {
        print_error_sources(source);
    }

    error!("{error}");
}

/// Prints a successful or error result, then exit.
///
/// A successful result is printed in JSON or YAML format, and any error encountered is recursively
/// sourced (backtrace). The program is then exited with an appropriate code. This helper cannot be called with
/// a result that doesn't contain data (`Result<()>`). Use [`exit_result`] in that case.
///
/// This helper should be consumed by commands at the end of their execution.
/// They can still return an early error that will be handled in `src/main.rs`.
///
/// See also [`exit_success`] and [`exit_result`] if you need to deal with other types.
pub fn exit_result_output<T: Debug + std::fmt::Display + Serialize>(res: Result<T, Error>) {
    match res {
        Ok(output) => match format(&output) {
            Ok(serialized) => {
                println!("{}", serialized);
                std::process::exit(0);
            }
            Err(e) => {
                print_error_sources(e);
                error!("An error occured during formatting output, see above.");
                std::process::exit(1)
            }
        },
        Err(e) => {
            print_error_sources(e);
            error!("An error occured, see backtrace above.");
            std::process::exit(1)
        }
    }
}

/// Prints a error result, if any, then exit the program.
///
/// A successful result is not printed, but any error encountered is recursively
/// sourced (backtrace). The program is then exited with an appropriate code. This helper cannot be called with
/// a result containing actual data. Use [`exit_result_output`] in that case.
///
/// This helper should be consumed by commands at the end of their execution.
/// They can still return an early error that will be handled in `src/main.rs`.
///
/// See also [`exit_success`] and [`exit_result_output`] if you need to deal with other types.
pub fn exit_result(res: Result<(), Error>) {
    if let Err(e) = res {
        print_error_sources(e);
        error!("An error occured, see backtrace above.");
        std::process::exit(1)
    }

    std::process::exit(0);
}

/// Prints a certain output then exit the program successfully.
///
/// A successful output is printed in JSON or YAML format. This helper cannot be called with
/// a `Result`. Use [`exit_result_output`] or [`exit_result`] in that case.
///
/// This helper should be consumed by commands at the end of their execution.
/// They can still return an early error that will be handled in `src/main.rs`.
///
/// See also [`exit_result`] and [`exit_result_output`] if you need to deal with other types.
pub fn exit_success<T: Debug + Serialize>(res: T) {
    match format(&res) {
        Ok(serialized) => {
            println!("{}", serialized);
            std::process::exit(0);
        }
        Err(e) => {
            print_error_sources(e);
            error!("An error occured during formatting output, see above.");
            std::process::exit(1)
        }
    }
}
