use snafu::prelude::*;

use std::ffi::OsStr;
use std::fs::metadata;
use std::os::unix::fs::MetadataExt;
use std::process::{Command, Output};

use crate::error::*;

/// Checks whether the current running program is running as root.
///
/// Panics when things are really fucked up and reading /proc fails.
pub fn is_root() -> bool {
    get_euid_self() == 0
}

/// Checks the current process effective user ID (EUID).
///
/// Panics when things are really fucked up and reading /proc fails.
pub fn get_euid_self() -> u32 {
    // UNWRAP NOTE: If reading /proc/self fails, there's no need to try anything. Just kill the program.
    metadata("/proc/self").map(|m| m.uid()).unwrap()
}

/// Runs a command with some arguments, and returns the [`Output`]. Does not error when the
/// command returns a non-zero exit code.
///
/// Errors when:
///   - `command` does not exist
///   - the program does not have permission to execute `command`
///   - out-of-memory and other terrible conditions????
///
/// Note: the error message may omit invalid UTF-8 characters from the output in order to display it.
/// TODO: Be more precise about the error if it's a Unicode error
/// TODO: Finally let's crash for UTF8 error for the moment but we should do something about it because it's not exactly uncommon in the wild
pub fn cmd<I, S>(command: &str, args: I) -> Result<Output, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    // This is not the most optimized, but we keep a copy of the arguments in case the command fails, so that we can
    // return a proper error.
    let args: Vec<String> = args
        .into_iter()
        .map(|x| x.as_ref().to_str().unwrap().to_string())
        .collect();
    Command::new(command)
        .args(&args)
        .output()
        .context(CmdSnafu {
            cmd: command.to_string(),
            args,
        })
}
