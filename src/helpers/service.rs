use std::process::Output;

use crate::helpers::process::cmd;

pub struct SystemCtl;

impl SystemCtl {
    /// Runs the `systemctl daemon-reload` command.
    ///
    /// Panics if running systemctl fails, not if the return code is non-zero.
    pub fn daemon_reload() -> Output {
        cmd("systemctl", vec!["daemon-reload"]).unwrap()
    }

    /// Runs the `systemctl enable` command, for a `unit`, with potential `extra` params. Returns true
    /// when the operation was successful (return code 0) and not aborted.
    ///
    /// Panics if running systemctl fails, not if the return code is non-zero.
    ///
    /// Example:
    ///
    /// ```rust
    /// use yunohost::helpers::service::SystemCtl;
    /// fn main() {
    ///     let res = Systemctl::enable("nginx", [ "--quiet", "--now" ]);
    ///     println!("Success {}", res.status.success());
    /// }
    /// ```
    pub fn enable(unit: &str, extra: &[&str]) -> bool {
        let mut args: Vec<&str> = vec!["enable"];
        args.extend(extra);
        args.push(unit);
        // UNWRAP NOTE: If systemctl fails to spawn, no need to continue anything...
        cmd("systemctl", &args).unwrap().status.success()
    }

    /// Runs the `systemctl disable` command, for a `unit`, with potential `extra` params. Returns true
    /// when the operation was successful (return code 0) and not aborted.
    ///
    /// Panics if running systemctl fails, not if the return code is non-zero.
    pub fn disable(unit: &str, extra: &[&str]) -> bool {
        let mut args: Vec<&str> = vec!["disable"];
        args.extend(extra);
        args.push(unit);
        // UNWRAP NOTE: If systemctl fails to spawn, no need to continue anything...
        cmd("systemctl", &args).unwrap().status.success()
    }

    /// Runs the `systemctl start` command, for a `unit`, with potential `extra` params. Returns true
    /// when the operation was successful (return code 0) and not aborted.
    ///
    /// Panics if running systemctl fails, not if the return code is non-zero.
    pub fn start(unit: &str, extra: &[&str]) -> bool {
        let mut args: Vec<&str> = vec!["start"];
        args.extend(extra);
        args.push(unit);
        // UNWRAP NOTE: If systemctl fails to spawn, no need to continue anything...
        cmd("systemctl", &args).unwrap().status.success()
    }

    /// Runs the `systemctl stop` command, for a `unit`, with potential `extra` params. Returns true
    /// when the operation was successful (return code 0) and not aborted.
    ///
    /// Panics if running systemctl fails, not if the return code is non-zero.
    pub fn stop(unit: &str, extra: &[&str]) -> bool {
        let mut args: Vec<&str> = vec!["stop"];
        args.extend(extra);
        args.push(unit);
        // UNWRAP NOTE: If systemctl fails to spawn, no need to continue anything...
        cmd("systemctl", &args).unwrap().status.success()
    }

    /// Checks whether a `unit` exists. You can use the long or short format for the unit name.
    ///
    /// Panics if running systemctl fails, not if the return code is non-zero.
    ///
    /// Example:
    ///
    /// ```rust
    /// use yunohost::helpers::service::SystemCtl;
    /// fn main() {
    ///     if Systemctl::exists("nginx") {
    ///         println!("nginx is installed");
    ///     }
    /// }
    /// ```
    pub fn exists(unit: &str) -> bool {
        let unit = if unit.contains('.') {
            unit.to_string()
        } else {
            format!("{unit}.")
        };

        // UNWRAP NOTE: If systemctl fails to spawn, no need to continue anything...
        let output = cmd(
            "systemctl",
            vec!["list-units", "--no-pager", "--plain", "--full"],
        )
        .unwrap();
        let output = String::from_utf8_lossy(&output.stdout);
        output.lines().any(|x| x.starts_with(&unit))
    }
}
