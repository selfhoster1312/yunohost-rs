use clap::{Parser, Subcommand};
use glob::glob;
use log::LevelFilter;
use snafu::prelude::*;

use yunohost::{
    error::*,
    helpers::{apt::*, file::*, process::*, service::*, user::*, group::*},
};

use std::fs::{copy, create_dir_all as mkdir, read_dir, write};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::exit;

// Static configuration, can be overriden at runtime (TODO)
const LOGIND_OVERRIDE: &'static str = include_str!("logind.service.override.conf");
const NFTABLES_OVERRIDE: &'static str = include_str!("nftables.service.override.conf");
const NTP_OVERRIDE: &'static str = include_str!("ntp.service.override.conf");

// Use me with complete name whatever.foo.d in /etc/systemd
pub struct SystemdOverride {
    dir: PathBuf,
}

impl SystemdOverride {
    fn service_dir_name(name: &str) -> String {
        if name.ends_with(".service.d") {
            name.to_string()
        } else if name.ends_with(".service") {
            format!("{name}.d")
        } else {
            format!("{name}.service.d")
        }
    }

    /// Handle ynh-override.conf for service *name* in `/etc/systemd/system`.
    ///
    /// For use in `pending_dir`, see [`Self::new_service_pending`].
    /// Acceptable `name` forms:
    /// - `SERVICE`
    /// - `SERVICE.service`
    /// - `SERVICE.service.d`
    pub fn new_service(name: &str) -> Self {
        let name = Self::service_dir_name(name.as_ref());

        Self {
            dir: PathBuf::from(&format!("/etc/systemd/system/{name}")),
        }
    }

    /// Handle ynh-override.conf for service *name* in `pending_dir`.
    ///
    /// For use on the OS globally, see [`Self::new_service`].
    /// Acceptable `name` forms:
    /// - `SERVICE`
    /// - `SERVICE.service`
    /// - `SERVICE.service.d`
    pub fn new_service_pending<U: AsRef<Path>>(name: &str, pending_dir: U) -> Self {
        let name = Self::service_dir_name(name);

        Self {
            dir: pending_dir
                .as_ref()
                .join(&format!("etc/systemd/system/{name}")),
        }
    }

    /// Handle ynh-override.conf for other systemd config *name*.
    ///
    /// For use in `pending_dir`, see [`Self::new_other_pending`].
    /// The foler `name` is appended without modification to `pending_dir/etc/systemd/`,
    /// then the ynh-override.conf is placed there.
    pub fn new_other(name: &str) -> Self {
        Self {
            dir: PathBuf::from(&format!("/etc/systemd/{name}")),
        }
    }

    /// Handle ynh-override.conf for other systemd config *name*.
    ///
    /// For use in OS globally, see [`Self::new_other`].
    /// The foler `name` is appended without modification to `/etc/systemd/`,
    /// then the ynh-override.conf is placed there.
    pub fn new_other_pending<U: AsRef<Path>>(name: &str, pending_dir: U) -> Self {
        Self {
            dir: pending_dir.as_ref().join(&format!("etc/systemd/{name}")),
        }
    }

    pub fn pre(&self, content: &str) {
        mkdir_p(&self.dir);
        write(self.dir.join("ynh-override.conf"), content).unwrap();
    }

    pub fn post(&self) {}
}

#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Clone, Debug, Subcommand)]
enum SubCommand {
    Init,
    Pre {
        #[arg()]
        files: PathBuf,
    },
    Post {
        #[arg()]
        files: Option<String>,
    },
}

fn do_init_regen() -> Result<(), Error> {
    if !is_root() {
        eprintln!("You need to be root to run this script");
        exit(1);
    }

    let conf_dir = PathBuf::from("/usr/share/yunohost/conf/yunohost");

    if !is_dir("/etc/yunohost") {
        mkdir("/etc/yunohost").unwrap();
    }

    // set default current_host
    if !is_file("/etc/yunohost/current_host") {
        write("/etc/yunohost/current_host", "yunohost.org").unwrap();
    }

    // copy default services and firewall
    if !is_file("/etc/yunohost/firewall.yml") {
        copy(conf_dir.join("firewall.yml"), "/etc/yunohost/firewall.yml").unwrap();
    }

    // allow users to access /media directory
    if !is_dir("/etc/skel/media") {
        mkdir("/media").unwrap();
        symlink("/media", "/etc/skel/media").unwrap();
    }

    // Cert folders
    mkdir_p("/etc/yunohost/certs");
    chown_recurse("/etc/yunohost/certs", "root", "ssl-cert");
    chmod("/etc/yunohost/certs", 0o750);

    // App folders
    mkdir_p("/etc/yunohost/apps");
    chmod("/etc/yunohost/apps", 0o700);
    mkdir_p("/home/yunohost.app");
    chmod("/home/yunohost.app", 0o755);

    // Domain settings
    mkdir_p("/etc/yunohost/domains");
    chmod("/etc/yunohost/domains", 0o700);

    // Backup folders
    mkdir_p("/home/yunohost.backup/archives");
    chmod("/home/yunohost.backup/archives", 0o750);
    chown("/home/yunohost.backup/archives", "root", Some("admins"));

    // Empty ssowat json persistent conf
    write("/etc/ssowat/conf.json.persistent", "{}").unwrap();
    chmod("/etc/ssowat/conf.json.persistent", 0o644);
    chown("/etc/ssowat/conf.json.persistent", "root", Some("root"));

    // Empty service conf
    // touch /etc/yunohost/services.yml

    mkdir_p("/var/cache/yunohost/repo");
    chown("/var/cache/yunohost", "root", Some("root"));
    chmod("/var/cache/yunohost", 0o700);

    copy(
        conf_dir.join("yunohost-api.service"),
        "/etc/systemd/system/yunohost-api.service",
    )
    .unwrap();
    copy(
        conf_dir.join("yunohost-firewall.service"),
        "/etc/systemd/system/yunohost-firewall.service",
    )
    .unwrap();
    copy(
        conf_dir.join("yunoprompt.service"),
        "/etc/systemd/system/yunoprompt.service",
    )
    .unwrap();

    SystemCtl::daemon_reload();
    SystemCtl::enable("yunohost-api.service", &["--quiet", "--now"]);

    // Yunohost-firewall is enabled only during postinstall, not init, not 100% sure why

    copy_to(conf_dir.join("dpkg-origins"), "/etc/dpkg/origins/yunohost");
    change_dpkg_vendor("/etc/dpkg/origins/yunohost")
        .context(ConfRegenYunohostInitDPKGVendorSnafu)?;

    Ok(())
}

fn do_pre_regen(pending_dir: PathBuf) -> Result<(), Error> {
    for dir in ["etc/systemd/system", "etc/cron.d", "etc/cron.daily"] {
        mkdir_p(pending_dir.join(dir));
    }

    // add cron job for diagnosis to be ran at 7h and 19h + a random delay between
    // 0 and 20min, meant to avoid every instances running their diagnosis at
    // exactly the same time, which may overload the diagnosis server.
    write(
        pending_dir.join("etc/cron.d/yunohost-diagnosis"),
        r#""
SHELL=/bin/bash
0 7,19 * * * root : YunoHost Automatic Diagnosis; sleep \$((RANDOM\\%1200)); yunohost diagnosis run --email > /dev/null 2>/dev/null || echo "Running the automatic diagnosis failed miserably"
        ""#,
    ).unwrap();

    // Cron job that upgrade the app list everyday
    write(
        pending_dir.join("etc/cron.daily/yunohost-fetch-apps-catalog"),
        r#""
#!/bin/bash
sleep \$((RANDOM%3600)); yunohost tools update apps > /dev/null
        ""#,
    )
    .unwrap();

    // Cron job that renew lets encrypt certificates if there's any that needs renewal
    write(
        pending_dir.join("etc/cron.daily/yunohost-certificate-renew"),
        r#""
#!/bin/bash
yunohost domain cert renew --email
        ""#,
    )
    .unwrap();

    // If we subscribed to a dyndns domain, add the corresponding cron
    // - delay between 0 and 60 secs to spread the check over a 1 min window
    // - do not run the command if some process already has the lock, to avoid queuing hundreds of commands...
    if glob("/etc/yunohost/dyndns/K*.key").unwrap().count() != 0 {
        write(
            pending_dir.join("etc/cron.d/yunohost-dyndns"),
            r#""
SHELL=/bin/bash
# Every 10 minutes,
#   - (sleep random 60 is here to spread requests over a 1-min window)
#   - if ip.yunohost.org answers ping (basic check to validate that we're connected to the internet and yunohost infra aint down)
#   - and if lock ain't already taken by another command
#   - trigger yunohost dyndns update
*/10 * * * * root : YunoHost DynDNS update; sleep \$((RANDOM\\%60)); ! ping -q -W5 -c1 ip.yunohost.org >/dev/null 2>&1 || test -e /var/run/moulinette_yunohost.lock || yunohost dyndns update >> /dev/null

            ""#,
        ).unwrap();
    } else {
        // (Delete cron if no dyndns domain found)
        write(pending_dir.join("etc/cron.d/yunohost-dyndns"), "").unwrap();
    }

    // Skip ntp if inside a container (inspired from the conf of systemd-timesyncd)
    if SystemCtl::exists("ntp.service") {
        let so = SystemdOverride::new_service_pending("ntp.service", &pending_dir);
        so.pre(NTP_OVERRIDE);
    }

    // Make nftable conflict with yunohost-firewall
    let so = SystemdOverride::new_service_pending("nftables.service", &pending_dir);
    so.pre(NFTABLES_OVERRIDE);

    // Don't suspend computer on LidSwitch
    let so = SystemdOverride::new_other_pending("logind.conf.d", &pending_dir);
    so.pre(LOGIND_OVERRIDE);

    let conf_dir = PathBuf::from("/usr/share/yunohost/conf/yunohost");

    for file in [
        "yunohost-api.service",
        "yunohost-firewall.service",
        "yunoprompt.service",
        "proc-hidepid.service",
    ] {
        copy(
            conf_dir.join(file),
            pending_dir.join(&format!("etc/systemd/system/{file}")),
        )
        .unwrap();
    }

    mkdir_p(pending_dir.join("etc/dpkg/origins"));
    copy(
        conf_dir.join("dpkg-origins"),
        pending_dir.join("etc/dpkg/origins/yunohost"),
    )
    .unwrap();

    Ok(())
}

fn do_post_regen(regen_conf_files: Option<String>) -> Result<(), Error> {
    // ######################
    // # Enfore permissions #
    // ######################

    chown("/home/yunohost.backup", "root", Some("admins"));
    chmod("/home/yunohost.backup", 0o700);

    chown("/home/yunohost.backup/archives", "root", Some("admins"));
    chmod("/home/yunohost.backup/archives", 0o770);

    chown("/var/cache/yunohost", "root", Some("root"));
    chmod("/var/cache/yunohost", 0o700);

    if path_exists("/var/www/.well-known/ynh-diagnosis") {
        chmod("/var/www/.well-known/ynh-diagnosis", 0o775);
    }

    // NB: x permission for 'others' is important for ssl-cert (and maybe mdns), otherwise slapd will fail to start because can't access the certs
    chmod("/etc/yunohost", 0o755);

    for path in glob("/etc/systemd/system/*.service").unwrap() {
        let path = path.unwrap();
        // Some are symlinks, in which case chown/chmod does not work
        if is_file(&path) {
            chown(&path, "root", Some("root"));
            chmod(&path, 0o644);
        }
    }

    for file in glob("/etc/php/*/fpm/pool.d/*.conf").unwrap() {
        let file = file.unwrap();
        chown(&file, "root", Some("root"));
        chmod(&file, 0o644);
    }

    // Certs
    // We do this with find because there could be a lot of them...
    chown_recurse("/etc/yunohost/certs", "root", "ssl-cert");
    chmod("/etc/yunohost/certs", 0o750);

    for path in glob("/etc/yunohost/certs/**/*").unwrap() {
        let path = path.unwrap();
        if path.is_file() {
            chmod(&path, 0o640);
        }
        if path.is_dir() {
            chmod(&path, 0o750);
        }
    }

    for path in glob("/etc/cron.*/yunohost-*").unwrap() {
        let path = path.unwrap();
        if path.is_file() {
            if path.parent().unwrap().file_name().unwrap() == "cron.d" {
                chmod(&path, 0o644);
            } else {
                chmod(&path, 0o755);
            }
            chown(&path, "root", Some("root"));
        }
    }

    for path in ["/var/www", "/var/log/nginx", "/etc/yunohost", "/etc/ssowat"] {
        let _ = cmd("setfacl", vec!["-m", "g:all_users:---", path])
            .context(ConfRegenYunohostPostSetfaclSnafu);
    }

    for user in YunohostUser::usernames().context(ConfRegenYunohostPostUsersSnafu)? {
        let home = format!("/home/{user}");
        if is_dir(&home) {
            let _ = cmd("setfacl", vec!["-m", "g:all_users:---", &home]);
        }
    }

    // Domain settings
    mkdir_p("/etc/yunohost/domains");

    // Misc configuration / state files
    for (file, do_chown) in read_dir("/etc/yunohost")
        .unwrap()
        .filter_map(|file| {
            let file = file.unwrap();
            let file = file.path();
            let name = file.file_name().unwrap();
            let extension = file.extension();

            match extension.map(|x| x.to_str().unwrap()) {
                Some("yaml") | Some("yml") | Some("json") => {
                    if name == "mdns.yml" {
                        Some((file, false))
                    } else {
                        Some((file, true))
                    }
                }
                None => {
                    if name == "mysql" || name == "psql" {
                        Some((file, true))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .collect::<Vec<(PathBuf, bool)>>()
    {
        if do_chown {
            chown(&file, "root", Some("root"));
        }
        chmod(&file, 0o600);
    }

    // Apps folder, custom hooks folder
    for path in [
        "/etc/yunohost/hooks.d",
        "/etc/yunohost/apps",
        "/etc/yunohost/domains",
    ] {
        chown(path, "root", None);
        chmod(path, 0o700);
    }

    // Create ssh.app and sftp.app groups if they don't exist yet
    YunohostGroup::ensure_exists("ssh.app")?;
    YunohostGroup::ensure_exists("sftp.app")?;

    if let Some(regen_conf_files) = regen_conf_files {
        // Propagates changes in systemd service config overrides
        let mut systemd_restart: Vec<String> = vec![];
        let mut systemd_enable: Vec<String> = vec![];
        let mut systemd_disable: Vec<String> = vec![];
        let mut systemd_reload = false;

        if SystemCtl::exists("ntp.service") {
            if regen_conf_files.contains("ntp.service.d/ynh-override.conf") {
                systemd_restart.push("ntp".into());
                systemd_reload = true;
            }
        }

        if regen_conf_files.contains("nftables.service.d/ynh-override.conf") {
            systemd_reload = true;
        }

        if regen_conf_files.contains("login.conf.d/ynh-override.conf") {
            systemd_reload = true;
            systemd_restart.push("systemd-logind".into());
        }

        if regen_conf_files.contains("yunohost-firewall.service")
            || regen_conf_files.contains("yunohost-api.service")
        {
            systemd_reload = true;
        }

        // TODO: disable proc-hidepid in containers??
        for entry in ["yunoprompt", "proc-hidepid"] {
            let service_file = format!("{entry}.service");
            if regen_conf_files.contains(&service_file) {
                systemd_reload = true;
                if path_exists(&format!("/etc/systemd/system/{service_file}")) {
                    systemd_enable.push(entry.into());
                } else {
                    systemd_disable.push(entry.into());
                }
            }
        }

        if systemd_reload {
            SystemCtl::daemon_reload();
        }

        if !systemd_disable.is_empty() {
            let mut args: Vec<String> = vec!["disable", "--quiet", "--now"]
                .into_iter()
                .map(|x| x.to_string())
                .collect();
            args.extend(systemd_disable);
            // TODO: maybe check exit code of systemctl? or use helper
            cmd("systemctl", args).context(ConfRegenYunohostPostSystemCtlDisableSnafu)?;
        }

        if !systemd_enable.is_empty() {
            let mut args: Vec<String> = vec!["enable", "--quiet", "--now"]
                .into_iter()
                .map(|x| x.to_string())
                .collect();
            args.extend(systemd_enable);
            // TODO: maybe check exit code of systemctl? or use helper
            cmd("systemctl", args).context(ConfRegenYunohostPostSystemCtlEnableSnafu)?;
        }

        if !systemd_restart.is_empty() {
            let mut args: Vec<String> = vec!["restart".to_string()];
            args.extend(systemd_restart);
            // TODO: maybe check exit code of systemctl? or use helper
            cmd("systemctl", args).context(ConfRegenYunohostPostSystemCtlRestartSnafu)?;
        }
    }

    change_dpkg_vendor("/etc/dpkg/origins/yunohost")
        .context(ConfRegenYunohostPreDPKGVendorSnafu)?;

    if path_exists("/etc/yunohost/installed") {
        ensure_file_remove("/etc/profile.d/check_yunohost_is_installed.sh")
            .context(ConfRegenYunohostPreIsInstalledCheckSnafu)?;
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    if cli.debug {
        pretty_env_logger::formatted_builder()
            .filter_level(LevelFilter::Debug)
            .init();
    } else {
        pretty_env_logger::formatted_builder()
            .filter_level(LevelFilter::Info)
            .init();
    }

    match cli.command {
        SubCommand::Init => do_init_regen(),
        SubCommand::Pre { files } => do_pre_regen(files),
        SubCommand::Post { files } => do_post_regen(files),
    }
}
