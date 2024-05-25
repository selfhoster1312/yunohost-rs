use clap::{Parser, Subcommand};
use glob::glob;
use log::LevelFilter;
use snafu::prelude::*;

use yunohost::{
    error::*,
    helpers::{apt::*, file::*, group::*, process::*, service::*, user::*},
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
        path(self.dir.to_str().unwrap()).mkdir_p().unwrap();
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

    if !path("/etc/yunohost").is_dir() {
        mkdir("/etc/yunohost").unwrap();
    }

    // set default current_host
    if !path("/etc/yunohost/current_host").is_file() {
        write("/etc/yunohost/current_host", "yunohost.org").unwrap();
    }

    // copy default services and firewall
    if !path("/etc/yunohost/firewall.yml").is_file() {
        copy(conf_dir.join("firewall.yml"), "/etc/yunohost/firewall.yml").unwrap();
    }

    // allow users to access /media directory
    if !path("/etc/skel/media").is_dir() {
        mkdir("/media").unwrap();
        symlink("/media", "/etc/skel/media").unwrap();
    }

    // Cert folders
    let p = path("/etc/yunohost/certs");
    p.mkdir_p().unwrap();
    p.chown_and_mode(0o750, "root", Some("ssl-cert")).unwrap();

    // App folders
    let p = path("/etc/yunohost/apps");
    p.mkdir_p().unwrap();
    p.chmod(0o700).unwrap();

    let p = path("/home/yunohost.app");
    p.chmod(0o755).unwrap();

    // Domain settings
    let p = path("/etc/yunohost/domains");
    p.chmod(0o700).unwrap();

    // Backup folders
    let p = path("/home/yunohost.backup/archives");
    p.mkdir_p().unwrap();
    p.chown_and_mode(0o750, "root", Some("admins")).unwrap();

    // Empty ssowat json persistent conf
    write("/etc/ssowat/conf.json.persistent", "{}").unwrap();
    path("/etc/ssowat/conf.json.persistent")
        .chown_and_mode(0o644, "root", Some("root"))
        .unwrap();

    // Empty service conf
    // touch /etc/yunohost/services.yml

    let p = path("/var/cache/yunohost/repo");
    p.mkdir_p().unwrap();
    p.chown_and_mode(0o700, "root", Some("root")).unwrap();

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

    let p = path(conf_dir.join("dpkg-origins").to_str().unwrap());
    p.copy_to(&path("/etc/dpkg/origins/yunohost")).unwrap();
    change_dpkg_vendor("/etc/dpkg/origins/yunohost")
        .context(ConfRegenYunohostInitDPKGVendorSnafu)?;

    Ok(())
}

fn do_pre_regen(pending_dir: PathBuf) -> Result<(), Error> {
    for dir in ["etc/systemd/system", "etc/cron.d", "etc/cron.daily"] {
        let dir = StrPath::from(pending_dir.join(dir).to_str().unwrap());
        dir.mkdir_p().unwrap();
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

    path("/etc/dpkg/origins").mkdir_p().unwrap();
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

    path("/home/yunohost.backup")
        .chown_and_mode(0o700, "root", Some("admins"))
        .unwrap();
    path("/home/yunohost.backup/archives")
        .chown_and_mode(0o700, "root", Some("admins"))
        .unwrap();
    path("/var/cache/yunohost")
        .chown_and_mode(0o700, "root", Some("root"))
        .unwrap();

    if path("/var/www/.well-known/ynh-diagnosis").exists() {
        path("/var/www/.well-known/ynh-diagnosis")
            .mode_set(0o775)
            .unwrap();
    }

    // NB: x permission for 'others' is important for ssl-cert (and maybe mdns), otherwise slapd will fail to start because can't access the certs
    path("/etc/yunohost").mode_set(0o755).unwrap();

    for p in glob("/etc/systemd/system/*.service").unwrap() {
        let p = StrPath::from(p.unwrap().to_str().unwrap());
        // Some are symlinks, in which case chown/chmod does not work
        if p.is_file() {
            p.chown_and_mode(0o644, "root", Some("root")).unwrap();
        }
    }

    for file in glob("/etc/php/*/fpm/pool.d/*.conf").unwrap() {
        let file = StrPath::from(file.unwrap().to_str().unwrap());

        file.chown_and_mode(0o644, "root", Some("root")).unwrap();
    }

    // Certs
    // We do this with find because there could be a lot of them...
    let p = path("/etc/yunohost/certs");
    p.chown_recurse("root", "ssl-cert").unwrap();
    p.chmod(0o750).unwrap();

    // TODO: readdir/glob integration
    for path in glob("/etc/yunohost/certs/**/*").unwrap() {
        let p = StrPath::from(path.unwrap().to_str().unwrap());
        if p.is_file() {
            p.chmod(0o640).unwrap();
        }
        if p.is_dir() {
            p.chmod(0o750).unwrap();
        }
    }

    for path in glob("/etc/cron.*/yunohost-*").unwrap() {
        let p = StrPath::from(path.unwrap().to_str().unwrap());
        if p.is_file() {
            if p.parent().unwrap().file_name().unwrap() == "cron.d" {
                p.chmod(0o644).unwrap();
            } else {
                p.chmod(0o755).unwrap();
            }
            p.chown("root", "root").unwrap();
        }
    }

    for path in ["/var/www", "/var/log/nginx", "/etc/yunohost", "/etc/ssowat"] {
        let _ = cmd("setfacl", vec!["-m", "g:all_users:---", path])
            .context(ConfRegenYunohostPostSetfaclSnafu);
    }

    for user in YunohostUser::usernames().context(ConfRegenYunohostPostUsersSnafu)? {
        let home = path(format!("/home/{user}"));
        if home.is_dir() {
            let _ = cmd("setfacl", vec!["-m", "g:all_users:---", home.as_str()]);
        }
    }

    // Domain settings
    path("/etc/yunohost/domains").mkdir_p().unwrap();

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
        let file = StrPath::from(file.to_str().unwrap());
        if do_chown {
            file.chown("root", "root").unwrap();
        }
        file.chmod(0o600).unwrap();
    }

    // Apps folder, custom hooks folder
    for path in [
        "/etc/yunohost/hooks.d",
        "/etc/yunohost/apps",
        "/etc/yunohost/domains",
    ] {
        let p = StrPath::from(path);
        p.chown_and_mode(0o700, "root", None).unwrap();
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
                if path(&format!("/etc/systemd/system/{service_file}")).exists() {
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

    if path("/etc/yunohost/installed").exists() {
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
